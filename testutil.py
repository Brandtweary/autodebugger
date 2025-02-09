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
    
    Args:
        logger: Logger instance to use
        result: Optional test result code
        
    Returns:
        True if all verifications pass, False otherwise
    """
    print("\nDEBUG: Running post-test verifications")
    print(f"DEBUG: Test result code: {result}")
    print(f"DEBUG: Failed tests: {logger.failed_tests}")
    print(f"DEBUG: Worker ID: {logger.worker_id}")
    print(f"DEBUG: Shared dir: {logger.shared_dir}")
    print(f"DEBUG: Current request: {logger.current_request.node.nodeid if logger.current_request else None}")
    
    # Print raw collector logs
    print("\nDEBUG: Raw collector logs:")
    print(logger.collector.logs)
    
    # Print filtered logs with different settings
    print("\nDEBUG: Filtered logs (default):")
    print(logger.get_filtered_logs())
    print("\nDEBUG: Filtered logs (show_info=True):")
    print(logger.get_filtered_logs(show_info=True))
    print("\nDEBUG: Filtered logs (no_capture=True):")
    print(logger.get_filtered_logs(no_capture=True))
    
    if result != 0:  # Only verify if there were test failures
        filtered_logs = logger.get_filtered_logs()
        # Check that each failed test has at least one log message
        for test_id in logger.failed_tests:
            if test_id not in filtered_logs:
                print(f"ERROR: Failed test {test_id} has no log messages")
                return False
            
    return True
