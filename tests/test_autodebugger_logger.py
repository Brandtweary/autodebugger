"""Tests for the autodebugger logger functionality."""
from typing import cast

import pytest
from _pytest.config import Config
from _pytest.fixtures import FixtureRequest
from _pytest.pytester import Pytester, RunResult

from autodebugger.autodebugger_logger import (
    AutodebuggerLogger,
    LogLevel,
    logger,
    LogEntry,
)


def test_log_writing(request):
    """Test that logs are properly written to files and can be read back."""
    import tempfile
    import os
    import json
    
    with tempfile.TemporaryDirectory() as tmp_dir:
        # Set up logger
        logger.set_shared_dir(tmp_dir)
        logger.set_worker_id("worker1")
        logger.set_current_request(request)
        
        # Write some logs
        logger.debug("Test debug")
        logger.error("Test error")
        
        # Sync logs to file
        logger.sync_logs()
        
        # Verify log file exists and contains correct data
        worker_dir = os.path.join(tmp_dir, "worker1")
        assert os.path.isdir(worker_dir)
        
        log_file = os.path.join(worker_dir, "logs.json")
        assert os.path.isfile(log_file)
        
        with open(log_file) as f:
            log_data = json.load(f)
            assert request.node.nodeid in log_data
            test_logs = log_data[request.node.nodeid]
            assert len(test_logs['messages']) == 2
            assert test_logs['messages'] == ["Test debug", "Test error"]
            assert test_logs['levels'] == [LogLevel.DEBUG.name, LogLevel.ERROR.name]


def test_log_filtering(request):
    """Test that get_filtered_logs correctly filters logs by level."""
    logger.set_current_request(request)
    test_id = request.node.nodeid
    
    try:
        # Add logs of different levels
        logger.debug("Debug message")
        logger.info("Info message")
        logger.warning("Warning message")
        logger.error("Error message")
        
        # For passed tests, only WARNING and above should be shown
        logs = logger.get_filtered_logs()
        assert test_id in logs
        assert len(logs[test_id]) == 2  # Only warning and error
        assert "WARNING: Warning message" in logs[test_id]
        assert "ERROR: Error message" in logs[test_id]
        assert "DEBUG: Debug message" not in logs[test_id]
        assert "INFO: Info message" not in logs[test_id]
        
        # For failed tests, all logs should be shown
        logger.failed_tests.add(test_id)
        logs = logger.get_filtered_logs()
        assert test_id in logs
        assert len(logs[test_id]) == 4  # All messages
        assert "DEBUG: Debug message" in logs[test_id]
        assert "INFO: Info message" in logs[test_id]
        assert "WARNING: Warning message" in logs[test_id]
        assert "ERROR: Error message" in logs[test_id]
        
        # Clean up failed test state
        logger.failed_tests.remove(test_id)
        
        # For info mode, INFO and above should be shown
        logs = logger.get_filtered_logs(show_info=True)
        assert test_id in logs
        assert len(logs[test_id]) == 3  # INFO, WARNING, and ERROR
        assert "DEBUG: Debug message" not in logs[test_id]
        assert "INFO: Info message" in logs[test_id]
        assert "WARNING: Warning message" in logs[test_id]
        assert "ERROR: Error message" in logs[test_id]
        
        # For no_capture mode, all logs should be shown even for passing tests
        logs = logger.get_filtered_logs(no_capture=True)
        assert test_id in logs
        assert len(logs[test_id]) == 4  # All messages
        assert "DEBUG: Debug message" in logs[test_id]
        assert "INFO: Info message" in logs[test_id]
        assert "WARNING: Warning message" in logs[test_id]
        assert "ERROR: Error message" in logs[test_id]
    finally:
        # Clean up
        logger.collector.clear()  # Clear all logs


def test_multiple_test_files(request):
    """Test that logs are properly collected from different test files."""
    import multiprocessing
    import tempfile
    import os
    import json
    
    with tempfile.TemporaryDirectory() as tmp_dir:
        # Set up logger in main process
        logger.set_shared_dir(tmp_dir)
        logger.set_worker_id(None)  # Main process
        
        def worker_process(test_file):
            """Worker process simulating a test file."""
            logger.set_shared_dir(tmp_dir)
            logger.set_worker_id(f"worker_{test_file}")
            
            # Create a test request
            class MockNode:
                @property
                def nodeid(self):
                    return f"tests/test_{test_file}.py::test_func"
            class MockRequest:
                @property
                def node(self):
                    return MockNode()
            request = MockRequest()
            
            # Write logs
            logger.set_current_request(request)
            logger.debug(f"Debug from {test_file}")
            logger.error(f"Error from {test_file}")
            logger.sync_logs()
        
        # Start worker processes for different test files
        workers = []
        for test_file in ["file1", "file2"]:
            p = multiprocessing.Process(
                target=worker_process,
                args=(test_file,)
            )
            workers.append(p)
            p.start()
        
        # Wait for workers to finish
        for p in workers:
            p.join()
        
        # Collect logs in main process
        logger.collect_worker_logs()
        
        # Verify logs from both test files
        all_logs = logger.collector.get_all_logs()
        assert "tests/test_file1.py::test_func" in all_logs
        assert "tests/test_file2.py::test_func" in all_logs
        
        # Check logs from file1
        log_data = all_logs["tests/test_file1.py::test_func"]
        assert "Debug from file1" in log_data["messages"]
        assert "Error from file1" in log_data["messages"]
        
        # Check logs from file2
        log_data = all_logs["tests/test_file2.py::test_func"]
        assert "Debug from file2" in log_data["messages"]
        assert "Error from file2" in log_data["messages"]


def test_singleton_behavior():
    """Test that AutodebuggerLogger is a true singleton."""
    instance1 = AutodebuggerLogger()
    instance2 = AutodebuggerLogger()
    assert instance1 is instance2


def test_multiprocess_logging(request):
    """Test that logger works correctly in a multiprocess environment.
    
    This test verifies that:
    1. Logs can be written from a worker process and read by the main process
    2. Log levels are preserved across process boundaries
    3. Failed tests are properly tracked and collected
    4. Each worker's logs are kept separate until merged
    """
    import multiprocessing
    import tempfile
    import os
    import json
    
    # Create a temporary directory for IPC
    with tempfile.TemporaryDirectory() as tmp_dir:
        # Set up logger in main process
        logger.set_shared_dir(tmp_dir)
        logger.set_worker_id(None)  # Main process
        
        def worker_process(worker_id):
            """Worker process that uses the logger."""
            # Set up logger in worker
            logger.set_shared_dir(tmp_dir)
            logger.set_worker_id(worker_id)
            
            # Create a test request
            class MockNode:
                @property
                def nodeid(self):
                    return f"test_{worker_id}"
            class MockRequest:
                @property
                def node(self):
                    return MockNode()
            request = MockRequest()
            
            # Use logger with different log levels
            logger.set_current_request(request)
            logger.debug(f"Debug from {worker_id}")
            logger.warning(f"Warning from {worker_id}")
            logger.error(f"Error from {worker_id}")
            
            # Mark test as failed for worker1
            if worker_id == "worker1":
                logger.failed_tests.add(f"test_{worker_id}")
            
            # Sync logs
            logger.sync_logs()
        
        # Start multiple worker processes
        workers = []
        for i in range(2):
            worker_id = f"worker{i+1}"
            p = multiprocessing.Process(
                target=worker_process,
                args=(worker_id,)
            )
            workers.append(p)
            p.start()
        
        # Wait for workers to finish
        for p in workers:
            p.join()
        
        # Collect logs in main process
        logger.collect_worker_logs()
        
        # Verify logs were collected from both workers
        all_logs = logger.collector.get_all_logs()
        assert "test_worker1" in all_logs
        assert "test_worker2" in all_logs
        
        # Verify log messages and levels for worker1
        log_data = all_logs["test_worker1"]  # LogEntry is already the correct type
        assert "Debug from worker1" in log_data["messages"]
        assert "Warning from worker1" in log_data["messages"]
        assert "Error from worker1" in log_data["messages"]
        assert LogLevel.DEBUG.name in log_data["levels"]
        assert LogLevel.WARNING.name in log_data["levels"]
        assert LogLevel.ERROR.name in log_data["levels"]
        
        # Verify log messages and levels for worker2
        log_data = all_logs["test_worker2"]  # LogEntry is already the correct type
        assert "Debug from worker2" in log_data["messages"]
        assert "Warning from worker2" in log_data["messages"]
        assert "Error from worker2" in log_data["messages"]
        assert LogLevel.DEBUG.name in log_data["levels"]
        assert LogLevel.WARNING.name in log_data["levels"]
        assert LogLevel.ERROR.name in log_data["levels"]
        
        # Verify failed test was tracked
        assert "test_worker1" in logger.failed_tests
        assert "test_worker2" not in logger.failed_tests
