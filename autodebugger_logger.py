"""
Logger module for autodebugger that provides test output capture and management.

This module provides a centralized logging system for test output that integrates
with pytest's output capture system. It stores logs during test execution and
can selectively display them based on test outcomes and verbosity settings.
"""

import logging
import sys
import os
import time
from dataclasses import dataclass, field
from enum import Enum
import pytest
from _pytest.config import Config
from _pytest.fixtures import FixtureRequest
from _pytest.logging import LogCaptureFixture
from contextlib import contextmanager
from typing import Dict, List, Optional, Set, Protocol
from multiprocessing import Queue
import multiprocessing
import queue
import json


class LogLevel(Enum):
    """Log levels matching Python's logging module."""
    DEBUG = logging.DEBUG
    INFO = logging.INFO
    WARNING = logging.WARNING
    ERROR = logging.ERROR
    CRITICAL = logging.CRITICAL


@dataclass
class LogEntry:
    """Container for log messages and their levels."""
    messages: List[str] = field(default_factory=list)
    levels: List[LogLevel] = field(default_factory=list)

    def add_log(self, message: str, level: LogLevel) -> None:
        """Add a log message with its level."""
        self.messages.append(message)
        self.levels.append(level)
    
    def to_dict(self) -> Dict[str, List]:
        """Convert to a serializable dict."""
        return {
            'messages': self.messages,
            'levels': [level.name for level in self.levels]
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, List]) -> 'LogEntry':
        """Create from a serialized dict."""
        entry = cls()
        entry.messages = data['messages']
        entry.levels = [LogLevel[name] for name in data['levels']]
        return entry


class LogCollector:
    """Collects and stores logs from test runs in memory.
    
    This class serves as an in-memory cache of logs that have been read from files.
    It is primarily used by get_filtered_logs to filter and format log messages
    based on verbosity settings and test status.
    """
    def __init__(self):
        self.logs: Dict[str, LogEntry] = {}

    def add_log(self, test_id: str, message: str, level: LogLevel) -> None:
        """Add a log message for a test."""
        if test_id not in self.logs:
            self.logs[test_id] = LogEntry()
        self.logs[test_id].add_log(message, level)

    def get_all_logs(self) -> Dict[str, Dict[str, List[str]]]:
        """Get all logs in dictionary format.
        
        This method is primarily used for testing. In production, logs are
        read directly from files in the shared directory.
        """
        return {test_id: entry.to_dict() for test_id, entry in self.logs.items()}

    def clear(self) -> None:
        """Clear all logs."""
        self.logs.clear()

    def merge_logs(self, test_id: str, entry: LogEntry) -> None:
        """Merge logs from another LogEntry."""
        if test_id not in self.logs:
            self.logs[test_id] = entry
        else:
            # Extend existing lists with new messages and levels
            self.logs[test_id].messages.extend(entry.messages)
            self.logs[test_id].levels.extend(entry.levels)


class AutodebuggerLogger:
    """Logger for autodebugger."""
    _instance = None
    worker_id = None  # Track which worker process this is
    failed_tests: Set[str] = set()
    shared_dir = None  # Shared directory for IPC

    def __new__(cls):
        if cls._instance is None:
            cls._instance = super().__new__(cls)
        return cls._instance

    def __init__(self):
        """Initialize the logger."""
        if getattr(self, 'initialized', False):
            return
        
        self.collector = LogCollector()
        self.current_request = None
        self.initialized = True

    def set_worker_id(self, worker_id: Optional[str]) -> None:
        """Set the worker ID for this process."""
        # If switching from worker to main process, sync any remaining logs
        if self.worker_id and worker_id is None:
            self.sync_logs()
        
        self.worker_id = worker_id

    def get_worker_id(self) -> Optional[str]:
        """Get the worker ID for this process."""
        return self.worker_id

    def set_current_request(self, request) -> None:
        """Set the current test request."""
        self.current_request = request

    def set_shared_dir(self, shared_dir: str) -> None:
        """Set the shared directory for IPC."""
        self.shared_dir = shared_dir

    def log(self, message: str, level: LogLevel) -> None:
        """Add a log message."""
        if self.current_request is None:
            return
            
        # Get test ID from request
        test_id = self.current_request.node.nodeid
        self.collector.add_log(test_id, message, level)
        # Note: Logs are synced after each test completes and at session end
        # See pytest_runtest_logreport and pytest_sessionfinish in conftest.py

    def debug(self, message: str) -> None:
        """Log a debug message."""
        self.log(message, LogLevel.DEBUG)

    def info(self, message: str) -> None:
        """Log an info message."""
        self.log(message, LogLevel.INFO)

    def warning(self, message: str) -> None:
        """Log a warning message."""
        self.log(message, LogLevel.WARNING)

    def error(self, message: str) -> None:
        """Log an error message."""
        self.log(message, LogLevel.ERROR)

    def critical(self, message: str) -> None:
        """Log a critical message."""
        self.log(message, LogLevel.CRITICAL)

    def get_all_logs(self) -> Dict[str, LogEntry]:
        """Get all logs."""
        return self.collector.logs

    def get_filtered_logs(self, no_capture: bool = False, show_info: bool = False) -> Dict[str, List[str]]:
        """Get filtered logs based on test status.
        
        Filtering rules:
        1. Failed tests: Show all logs (including DEBUG)
        2. Passed tests with no_capture: Show all logs
        3. Passed tests with show_info: Show INFO and above
        4. Passed tests: Show only WARNING and above
        """
        filtered = {}
        for test_id, entry in self.collector.logs.items():
            # Show all logs for failed tests or when in no_capture mode
            show_all = test_id in self.failed_tests or no_capture
            
            # Filter messages based on level and test status
            messages = []
            for msg, level in zip(entry.messages, entry.levels):
                if (show_all or 
                    level in (LogLevel.WARNING, LogLevel.ERROR, LogLevel.CRITICAL) or
                    (show_info and level == LogLevel.INFO)):
                    messages.append(f"{level.name}: {msg}")
            
            if messages:
                filtered[test_id] = messages
                
        return filtered

    def print_test_logs(self, no_capture: bool = False, show_info: bool = False) -> None:
        """Print all test logs in a formatted way.
        
        Args:
            no_capture: If True, show all log levels for all tests, similar to pytest's -s flag
            show_info: If True, show INFO and above for all tests, similar to pytest's -i flag
        
        Format:
        ================================== test logs ==================================
        
        test_name.py::test_function FAILED
            ERROR: error message
            WARNING: warning message
            
        test_name2.py::test_function2 PASSED
            WARNING: warning message
        """
        filtered_logs = self.get_filtered_logs(no_capture=no_capture, show_info=show_info)
        if not filtered_logs:
            return
            
        print("\n================================== test logs ==================================\n")
        
        for test_id in sorted(filtered_logs.keys()):
            # Show test status
            status = "FAILED" if test_id in self.failed_tests else "PASSED"
            print(f"{test_id} {status}")
            
            # Show logs indented
            for msg in filtered_logs[test_id]:
                print(f"    {msg}")
            print("")  # Empty line between tests

    def sync_logs(self) -> None:
        """Sync logs from worker to main process."""
        if not self.shared_dir or not self.worker_id:
            return
        
        logs = {
            test_id: entry.to_dict()
            for test_id, entry in self.collector.logs.items()
        }
        
        # Write logs to file
        import os
        
        # Verify both paths are strings
        if not isinstance(self.worker_id, str):
            raise ValueError("worker_id must be a string")
        if not isinstance(self.shared_dir, str):
            raise ValueError("shared_dir must be a string")
            
        # Create worker directory if it doesn't exist
        worker_dir = os.path.join(self.shared_dir, self.worker_id)
        os.makedirs(worker_dir, exist_ok=True)
        
        # Write logs to file
        log_file = os.path.join(worker_dir, 'logs.json')
        with open(log_file, 'w') as f:
            json.dump(logs, f)
            
        # Write failed tests to file
        failed_tests_file = os.path.join(worker_dir, 'failed_tests.json')
        with open(failed_tests_file, 'w') as f:
            json.dump(list(self.failed_tests), f)

    def collect_worker_logs(self) -> None:
        """Collect logs from all worker processes."""
        if not self.shared_dir:
            return
        
        import os
        import json
        
        # Clear existing logs and failed tests
        self.collector.clear()
        self.failed_tests.clear()
        
        # Collect logs and failed tests from each worker directory
        for worker_dir in os.listdir(self.shared_dir):
            worker_path = os.path.join(self.shared_dir, worker_dir)
            if not os.path.isdir(worker_path):
                continue
                
            # Read logs
            log_file = os.path.join(worker_path, 'logs.json')
            if os.path.exists(log_file):
                with open(log_file) as f:
                    logs = json.load(f)
                    for test_id, log_data in logs.items():
                        entry = LogEntry.from_dict(log_data)
                        self.collector.merge_logs(test_id, entry)
            
            # Read failed tests
            failed_tests_file = os.path.join(worker_path, 'failed_tests.json')
            if os.path.exists(failed_tests_file):
                with open(failed_tests_file) as f:
                    failed_tests = json.load(f)
                    self.failed_tests.update(failed_tests)

# Global logger instance (one per process)
logger = AutodebuggerLogger()


class TestNode(Protocol):
    """Protocol for test node objects."""
    @property
    def nodeid(self) -> str: ...


class TestRequest(Protocol):
    """Protocol for test request objects."""
    @property
    def node(self) -> TestNode: ...


@contextmanager
def autodebugger_logger_context(request: FixtureRequest):
    """Automatically set the current test context for logging."""
    logger.set_current_request(request)
    try:
        yield
    finally:
        logger.set_current_request(None)


@pytest.fixture(autouse=True)
def autodebugger_logger_fixture():
    return autodebugger_logger_context


def pytest_configure(config: Config) -> None:
    """Configure pytest plugin."""
    config.pluginmanager.register(AutodebuggerLogger(), "autodebugger_logger")
