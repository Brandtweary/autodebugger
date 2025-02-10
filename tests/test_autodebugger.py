"""
Tests for the autodebugger test runner.
"""
import os
import uuid
from pathlib import Path
import pytest

from autodebugger.testutil import generate_test_dir
from autodebugger.autodebugger_logger import logger


def test_generate_test_dir(request):
    """Test that test directory paths are correctly generated."""
    logger.set_current_request(request)
    logger.info("Testing directory generation with default arguments")
    
    # Test with default arguments
    path = generate_test_dir()
    logger.debug(f"Generated path: {path}")
    assert isinstance(path, Path)
    assert path.parent == Path("/tmp")
    assert path.name.startswith("test_")
    assert len(path.name) > 10  # Should include UUID
    
    # Test with custom base directory
    logger.debug("Testing with custom base directory /var/tmp")
    base_dir = Path("/var/tmp")
    path = generate_test_dir(base_dir=base_dir)
    logger.debug(f"Generated path with custom base: {path}")
    assert path.parent == base_dir
    
    # Test with prefix
    logger.debug("Testing with custom prefix 'mytest'")
    path = generate_test_dir(prefix="mytest")
    logger.debug(f"Generated path with prefix: {path}")
    assert path.name.startswith("testmytest_")
    
    # Test with special characters in prefix
    logger.debug("Testing with special characters in prefix")
    path = generate_test_dir(prefix="my/test with spaces")
    logger.debug(f"Generated path with special chars: {path}")
    assert "/" not in path.name
    assert " " not in path.name


def test_command_parsing(request):
    """Test that command line arguments are correctly parsed."""
    logger.set_current_request(request)
    logger.debug("Testing command line argument parsing")
    
    from autodebugger.cli import split_pytest_args
    
    test_cases = [
        # Basic case with test path
        {
            "args": ["tests/test_file.py"],
            "expected": {
                "autodebugger": ["tests/test_file.py"],
                "pytest": []
            }
        },
        # Mixed pytest and autodebugger args
        {
            "args": ["tests", "-v", "-n", "4", "-k", "test_pattern"],
            "expected": {
                "autodebugger": ["tests"],
                "pytest": ["-v", "-n", "4", "-k", "test_pattern"]
            }
        },
        # Pytest args with values
        {
            "args": ["--cov=src", "-k", "test_pattern", "--tb=short"],
            "expected": {
                "autodebugger": [],
                "pytest": ["--cov=src", "-k", "test_pattern", "--tb=short"]
            }
        },
        # Complex mix of args
        {
            "args": [
                "tests/test_file.py",
                "-n", "4",
                "-v",
                "--cov=src",
                "-k", "test_pattern",
                "--capture=no"
            ],
            "expected": {
                "autodebugger": ["tests/test_file.py"],
                "pytest": ["-n", "4", "-v", "--cov=src", "-k", "test_pattern", "--capture=no"]
            }
        },
        # Edge case with multiple test paths
        {
            "args": [
                "tests/test_1.py",
                "tests/test_2.py",
                "-v",
                "-n", "0"
            ],
            "expected": {
                "autodebugger": ["tests/test_1.py", "tests/test_2.py"],
                "pytest": ["-v", "-n", "0"]
            }
        },
        # Test autodebugger -i flag
        {
            "args": ["tests/test_file.py", "-v", "-i"],
            "expected": {
                "autodebugger": ["tests/test_file.py", "-i"],
                "pytest": ["-v"]
            }
        },
        # Test autodebugger --info flag after another flag
        {
            "args": ["tests/test_file.py", "-v", "--info"],
            "expected": {
                "autodebugger": ["tests/test_file.py", "--info"],
                "pytest": ["-v"]
            }
        }
    ]
    
    for i, case in enumerate(test_cases):
        logger.debug(f"Testing case {i + 1}: {case['args']}")
        autodebugger_args, pytest_args = split_pytest_args(case["args"])
        logger.debug(f"Split result - autodebugger: {autodebugger_args}, pytest: {pytest_args}")
        assert autodebugger_args == case["expected"]["autodebugger"], \
            f"Case {i}: Autodebugger args mismatch.\nExpected: {case['expected']['autodebugger']}\nGot: {autodebugger_args}"
        assert pytest_args == case["expected"]["pytest"], \
            f"Case {i}: Pytest args mismatch.\nExpected: {case['expected']['pytest']}\nGot: {pytest_args}"

def test_always_fails(request):
    """Test that always fails to help debug log collection."""
    from autodebugger.autodebugger_logger import logger
    logger.set_current_request(request)
    logger.debug("About to fail")
    logger.error("This is an error message that should be collected")
    assert False, "This test fails on purpose to debug log collection"