"""Configure pytest for autodebugger."""

import pytest
from pytest import Config
from _pytest.nodes import Item
from _pytest.runner import CallInfo
from pathlib import Path
from typing import Optional, TYPE_CHECKING, cast, Protocol
from multiprocessing import Manager
import tempfile
import os
import shutil
import sys
import time
import json

if TYPE_CHECKING:
    class ConfigWithSharedDir(Protocol):
        """Type protocol for Config with our custom shared_dir attribute."""
        shared_dir: str
        def addinivalue_line(self, name: str, line: str) -> None: ...
        def getoption(self, name: str) -> bool: ...

from autodebugger.autodebugger_logger import logger
from autodebugger.testutil import generate_test_dir

# Registration flags to prevent double registration
_registered = False
_registered_nodes = set()
_session_finished = False

@pytest.fixture
def test_dir_name(request):
    """Generate a unique test directory name.
    
    The directory will be created by tmp_path fixture.
    Uses the test name as a prefix for easier debugging.
    """
    test_name = request.node.name.replace("[", "").replace("]", "")
    return generate_test_dir(prefix=test_name)

def pytest_configure(config: Config):
    """Configure pytest. ONLY for main process setup."""
    global _registered
    if _registered:
        return
    _registered = True
        
    # Skip if this is a worker process
    if hasattr(config, 'workerinput'):
        return
        
    # Only create shared directory if it doesn't exist
    if not logger.get_shared_dir():
        shared_dir = tempfile.mkdtemp(prefix='test_')
        print(f"[DEBUG] Creating new shared_dir={shared_dir}")
        logger.register(shared_dir=shared_dir)


@pytest.hookimpl(tryfirst=True)
def pytest_configure_node(node):
    """Configure a worker node."""
    global _registered_nodes
    node_id = id(node)
    if node_id in _registered_nodes:
        return
    _registered_nodes.add(node_id)
    
    # Get shared directory
    shared_dir = logger.get_shared_dir()
    
    if shared_dir:
        # Make worker ID unique by adding timestamp and gateway ID
        worker_id = f"{node.gateway.id}_{int(time.time() * 1000)}"
        node.workerinput['shared_dir'] = shared_dir
        node.workerinput['worker_id'] = worker_id


def pytest_sessionstart(session):
    """Called when test session starts. ONLY for worker process setup."""
    # Skip if this is the main process
    if not hasattr(session.config, 'workerinput'):
        return
        
    # Get shared directory from worker input
    shared_dir = session.config.workerinput.get('shared_dir')
    worker_id = session.config.workerinput.get('worker_id')
    
    # Register logger with worker info
    if shared_dir and worker_id:
        logger.register(shared_dir=shared_dir, worker_id=worker_id)


def pytest_runtest_logreport(report):
    """Process test report."""
    if not hasattr(report, 'nodeid'):
        return
        
    if report.failed:
        logger.failed_tests.add(report.nodeid)
        
    # Only sync logs after the test has finished running
    if report.when == "call":
        logger.sync_logs()


def pytest_sessionfinish(session):
    """Clean up after test session."""
    global _session_finished
    
    # Only collect logs once per test session
    if _session_finished:
        return
    _session_finished = True
    
    # Only collect logs in main process
    if not hasattr(session.config, "workerinput"):
        shared_dir = logger.get_shared_dir()
        if not shared_dir:
            print("[WARNING] No shared directory found")
            return
        
        # Verify all registered workers have logs
        worker_dirs = {d for d in os.listdir(shared_dir) if os.path.isdir(os.path.join(shared_dir, d))}
        
        # Group directories by worker ID (gw0, gw1, etc)
        worker_logs = {}
        for d in worker_dirs:
            worker_id = d.split('_')[0]  # gw0_timestamp -> gw0
            log_file = os.path.join(shared_dir, d, 'logs.json')
            if os.path.exists(log_file):
                with open(log_file) as f:
                    logs = json.load(f)
                    if worker_id not in worker_logs:
                        worker_logs[worker_id] = set()
                    worker_logs[worker_id].update(logs.keys())
        
        # Collect and print logs
        logger.collect_worker_logs()
    
    # In worker process, sync logs before finishing
    else:
        logger.sync_logs()
