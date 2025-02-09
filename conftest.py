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

if TYPE_CHECKING:
    class ConfigWithSharedDir(Protocol):
        """Type protocol for Config with our custom shared_dir attribute."""
        shared_dir: str
        def addinivalue_line(self, name: str, line: str) -> None: ...
        def getoption(self, name: str) -> bool: ...

from autodebugger.autodebugger_logger import logger
from autodebugger.testutil import generate_test_dir

# Track if we've already registered autodebugger fixtures
AUTODEBUGGER_REGISTERED = False

@pytest.fixture
def test_dir_name(request):
    """Generate a unique test directory name.
    
    The directory will be created by tmp_path fixture.
    Uses the test name as a prefix for easier debugging.
    """
    test_name = request.node.name.replace("[", "").replace("]", "")
    return generate_test_dir(prefix=test_name)


def pytest_configure(config: Config) -> None:
    """Configure pytest."""
    global AUTODEBUGGER_REGISTERED
    
    # Always allow registration in worker processes
    if hasattr(config, 'workerinput'):
        AUTODEBUGGER_REGISTERED = False
    elif AUTODEBUGGER_REGISTERED:
        return
        
    AUTODEBUGGER_REGISTERED = True
    
    # Register custom markers
    config.addinivalue_line(
        "markers",
        "log_level(level): Set the log level for a specific test"
    )
    
    # Set up autodebugger logger
    logger.set_shared_dir(str(Path(config.rootpath) / ".pytest_cache"))
    
    # Set worker ID for xdist
    worker_id = os.environ.get('PYTEST_XDIST_WORKER')
    logger.set_worker_id(worker_id)
    
    # Initialize shared directory in main process only
    if not hasattr(config, 'workerinput'):  # We're in the main process
        import tempfile
        shared_dir = tempfile.mkdtemp()
        setattr(config, 'shared_dir', shared_dir)
        logger.set_shared_dir(shared_dir)
        logger.set_worker_id(None)  # Ensure we're in main process mode


@pytest.hookimpl(tryfirst=True)
def pytest_configure_node(node):
    """Configure each worker node."""
    # Pass shared directory to worker
    node.workerinput['shared_dir'] = getattr(node.config, 'shared_dir')


def pytest_sessionstart(session):
    """Called when test session starts."""
    # In worker process, get shared directory from worker input
    if hasattr(session.config, 'workerinput'):
        logger.set_shared_dir(session.config.workerinput['shared_dir'])
        logger.set_worker_id(session.config.workerinput['workerid'])


def pytest_runtest_logreport(report):
    """Handle test reports for logging."""
    if report.when == 'call':  # Only process the actual test call
        # Track failed tests in worker process
        if logger.worker_id and report.failed:
            logger.failed_tests.add(report.nodeid)
            
        # Sync logs after each test
        try:
            logger.sync_logs()
        except Exception as e:
            print(f"Error syncing logs: {e}")


def pytest_sessionfinish(session):
    """Called when test session has finished."""
    try:
        # Sync any remaining logs before shutdown
        logger.sync_logs()
        
        # In main process, collect all worker logs and clean up
        if logger.worker_id is None:  # We're in the main process
            logger.collect_worker_logs()
            
            # Clean up shared directory
            import shutil
            shutil.rmtree(getattr(session.config, 'shared_dir'))
    except Exception as e:
        print(f"Error in session finish: {e}")
