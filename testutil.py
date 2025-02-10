"""
Test utilities for managing temporary directories and other test resources.
"""
import os
import uuid
from pathlib import Path
from typing import Optional
import json

def generate_test_dir(base_dir: Optional[Path] = None, prefix: str = "") -> Path:
    """Generate a unique temporary directory path for tests.
    
    This function only generates the path - it does not create the directory.
    The directory will be created by pytest's tmp_path fixture.
    
    Args:
        base_dir: Optional base directory. If None, uses system temp directory
        prefix: Optional prefix for the directory name
        
    Returns:
        Path object for the generated directory
    """
    unique_id = str(uuid.uuid4())[:8]
    safe_prefix = prefix.replace(" ", "").replace("/", "").replace("\\", "")
    dir_name = f"test{safe_prefix}_{unique_id}" if safe_prefix else f"test_{unique_id}"
    
    if base_dir is None:
        base_dir = Path(os.getenv("PYTEST_BASE_TEMP", "/tmp"))
    
    return base_dir / dir_name


def run_post_test_verifications(logger, result: Optional[int] = None) -> bool:
    """Run post-test verifications."""
    # Verify logger was configured
    if logger.get_shared_dir() is None:
        print("[ERROR] Logger shared_dir was not set")
        return False
    
    # Verify test logs were collected
    if not logger.collector.logs:
        print("[ERROR] No test logs were collected")
        return False
        
    return True


def ensure_conftest_exists(test_paths):
    """Ensure a conftest.py exists in the current directory if needed.
    
    This function:
    1. Checks if we're in the autodebugger package (no action needed)
    2. Otherwise creates conftest.py in the working directory
    3. Warns about any existing conftest.py files in test paths
    
    Args:
        test_paths: List of paths to test directories/files
    """
    cwd = Path.cwd()
    
    # Skip if we're in the autodebugger package itself
    autodebugger_dir = Path(__file__).parent.resolve()
    if cwd.resolve() == autodebugger_dir.resolve():
        print("autodebugger: In autodebugger package, skipping conftest creation")
        return
    
    # Check for existing conftest.py files in test paths
    existing_conftests = []
    for test_path in test_paths:
        path = Path(test_path)
        # If it's a file, check its directory
        if path.is_file():
            path = path.parent
        # Check this directory and all parents up to cwd
        while path != cwd and path != path.parent:
            conftest = path / "conftest.py"
            if conftest.exists() and conftest.resolve() != autodebugger_dir / "conftest.py":
                existing_conftests.append(conftest)
            path = path.parent
    
    # Warn about existing conftests
    if existing_conftests:
        print("\nautodebugger: Warning - Found existing conftest.py files that may conflict:")
        for conftest in existing_conftests:
            print(f"  - {conftest}")
        print("  These may interfere with autodebugger's logging. If you experience issues,")
        print("  consider removing them or ensuring they properly import autodebugger's fixtures.")
    
    # Create conftest.py in current directory if it doesn't exist
    conftest_path = cwd / "conftest.py"
    if not conftest_path.exists():
        conftest_content = '''"""Root conftest.py that imports and re-exports autodebugger fixtures."""

import sys
from pathlib import Path
import pytest

# Track if we've already registered autodebugger fixtures
AUTODEBUGGER_REGISTERED = False

# Import autodebugger's conftest hooks and fixtures
from autodebugger.conftest import (
    pytest_configure as autodebugger_configure,
    pytest_configure_node,
    pytest_sessionstart,
    pytest_runtest_logreport,
    pytest_sessionfinish,
    test_dir_name,  # Re-export the fixture
)

def pytest_configure(config):
    """Configure pytest with autodebugger fixtures.
    
    We use a global flag to prevent double registration of fixtures
    and hooks when pytest discovers both this conftest and autodebugger's.
    """
    global AUTODEBUGGER_REGISTERED
    if AUTODEBUGGER_REGISTERED:
        return
        
    # Call autodebugger's configure function
    autodebugger_configure(config)
    AUTODEBUGGER_REGISTERED = True

# Re-export the other hooks by making them available in this module's namespace
__all__ = [
    'pytest_configure_node',
    'pytest_sessionstart',
    'pytest_runtest_logreport',
    'pytest_sessionfinish',
    'test_dir_name',
]
'''
        with open(conftest_path, 'w') as f:
            f.write(conftest_content)
        print(f"\nautodebugger: Created {conftest_path} for test logging")
