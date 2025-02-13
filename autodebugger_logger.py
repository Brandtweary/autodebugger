"""
Logger module for autodebugger that provides test output capture and management.

This module provides a centralized logging system for test output that integrates
with pytest's output capture system. It stores logs during test execution and
can selectively display them based on test outcomes and verbosity settings.

Important: Only use logging within test functions, not in fixtures. Logs in fixtures
will not be captured since they are not associated with a specific test. If you need
to log setup information, do it at the start of each test function.

Example usage in tests:
```python
from autodebugger.autodebugger_logger import logger

def test_my_feature(request):
    # Associate logs with this test - required at the start of each test
    logger.set_current_request(request)
    
    # Log messages at different levels
    logger.debug("Setting up test data...")
    logger.info("Processing feature X")
    
    result = process_data()
    if result.has_warnings:
        logger.warning(f"Warning in data: {result.warning_msg}")
    
    try:
        assert result.is_valid
    except AssertionError:
        logger.error(f"Validation failed: {result.error_details}")
        raise
```

The logs will be displayed based on test outcome and verbosity:
- Failed tests show all log levels
- Passed tests show WARNING and above by default
- Use autodebugger -i to see INFO logs
- Use autodebugger -s to see all logs including DEBUG
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
from multiprocessing import Manager, Queue
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
    """Logger for the autodebugger framework.

    The logger collects logs from multiple test processes and combines them in the main
    process for display. It handles both local and distributed test execution.

    Note on Testing Strategy:
    The logger is intentionally tested through integration rather than unit tests.
    This is because it is a stateful singleton that must coordinate log collection
    across multiple processes when running with pytest-xdist. Attempting to test it
    in isolation leads to test pollution, as the logger's shared state (worker IDs,
    shared directories, etc.) affects other tests running in parallel.

    Instead, we verify the logger's behavior through actual usage in the test suite,
    where we can observe that:
    1. Logs are properly collected from worker processes
    2. Failed tests show all log levels
    3. Passing tests filter logs appropriately
    """
    
    def __init__(self):
        """Initialize logger."""
        self.collector = LogCollector()
        self.failed_tests = set()
        self.shared_dir = None
        self.worker_id = None

    def register(self, shared_dir: str | None = None, worker_id: str | None = None):
        """Register logger with shared directory and worker id."""
        if shared_dir:
            self.shared_dir = shared_dir
        if worker_id:
            self.worker_id = worker_id
            
    def get_shared_dir(self) -> str | None:
        """Get shared directory for current process."""
        return self.shared_dir

    def set_current_request(self, request) -> None:
        """Set the current test request."""
        self.current_request = request

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
        shared_dir = self.get_shared_dir()
        
        # Main process doesn't need to sync - it collects from workers
        if not self.worker_id:
            return
            
        # Worker must have shared_dir
        if not shared_dir:
            return
        
        logs = {
            test_id: entry.to_dict()
            for test_id, entry in self.collector.logs.items()
        }
        
        # Verify both paths are strings
        if not isinstance(self.worker_id, str):
            raise ValueError("worker_id must be a string")
        if not isinstance(shared_dir, str):
            raise ValueError("shared_dir must be a string")
            
        # Create worker directory if it doesn't exist
        worker_dir = os.path.join(shared_dir, self.worker_id)
        os.makedirs(worker_dir, exist_ok=True)
        
        # Write logs to file
        log_file = os.path.join(worker_dir, 'logs.json')
        with open(log_file, 'w') as f:
            json.dump(logs, f)
        
        # Write failed tests to file
        failed_file = os.path.join(worker_dir, 'failed.json')
        with open(failed_file, 'w') as f:
            json.dump(list(self.failed_tests), f)
        
    def collect_worker_logs(self) -> None:
        """Collect logs from all worker processes."""
        shared_dir = self.get_shared_dir()
        if not shared_dir:
            return
        
        # Clear existing logs before collecting
        self.collector.clear()
        self.failed_tests.clear()
        
        # Collect logs and failed tests from each worker directory
        for worker_dir in os.listdir(shared_dir):
            worker_path = os.path.join(shared_dir, worker_dir)
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
            failed_tests_file = os.path.join(worker_path, 'failed.json')
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
