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


def run_post_test_verifications(logger, result: Optional[int] = None) -> bool:
    """Run post-test verifications
    
    Verifies that:
    1. The conftest.py is properly loaded (by checking worker_id and shared_dir)
    2. Log collection is working (by checking collector logs)
    
    Args:
        logger: Logger instance to use
        result: Optional test result code
        
    Returns:
        True if all verifications pass, False otherwise
    """
    print("\nDEBUG: Running post-test verifications")
    
    # 1. Verify conftest.py is loaded by checking worker setup
    print(f"DEBUG: Worker ID: {logger.worker_id}")
    print(f"DEBUG: Shared dir: {logger.shared_dir}")
    
    if logger.shared_dir is None:
        print("ERROR: shared_dir is None - conftest.py not loaded properly")
        return False
    
    # 2. Verify log collection is working
    print("\nDEBUG: Raw collector logs:")
    print(logger.collector.logs)
    
    # In worker processes, we should have logs and they should be syncing
    if logger.worker_id is not None and not logger.collector.logs:
        print("ERROR: No logs collected in worker process")
        return False
        
    return True
