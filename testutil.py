"""
Test utilities for managing temporary directories and other test resources.
"""
import os
import uuid
from pathlib import Path
from typing import Optional

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


def pytest_configure(config):
    """Pytest hook to set up global test configuration.
    
    This runs before any tests are collected.
    """
    # Create base temp directory if it doesn't exist
    base_temp = os.getenv("PYTEST_BASE_TEMP")
    if base_temp:
        Path(base_temp).mkdir(parents=True, exist_ok=True)
