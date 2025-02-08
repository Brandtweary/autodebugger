"""
Tests for the autodebugger test runner.
"""
import os
import uuid
from pathlib import Path
import pytest

from autodebugger.testutil import generate_test_dir


def test_generate_test_dir():
    """Test that test directory paths are correctly generated."""
    # Test with default arguments
    path = generate_test_dir()
    assert isinstance(path, Path)
    assert path.parent == Path("/tmp")
    assert path.name.startswith("test_")
    assert len(path.name) > 10  # Should include UUID
    
    # Test with custom base directory
    base_dir = Path("/var/tmp")
    path = generate_test_dir(base_dir=base_dir)
    assert path.parent == base_dir
    
    # Test with prefix
    path = generate_test_dir(prefix="mytest")
    assert path.name.startswith("testmytest_")
    
    # Test with special characters in prefix
    path = generate_test_dir(prefix="my/test with spaces")
    assert "/" not in path.name
    assert " " not in path.name


def test_command_parsing():
    """Test that command line arguments are correctly parsed."""
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
        }
    ]
    
    for i, case in enumerate(test_cases):
        autodebugger_args, pytest_args = split_pytest_args(case["args"])
        assert autodebugger_args == case["expected"]["autodebugger"], \
            f"Case {i}: Autodebugger args mismatch.\nExpected: {case['expected']['autodebugger']}\nGot: {autodebugger_args}"
        assert pytest_args == case["expected"]["pytest"], \
            f"Case {i}: Pytest args mismatch.\nExpected: {case['expected']['pytest']}\nGot: {pytest_args}"
