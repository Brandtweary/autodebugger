"""
Command-line interface for the autodebugger.

The autodebugger is a test runner that enhances pytest with improved test isolation
and directory management. It ensures each test gets a unique temporary directory
and handles cleanup automatically.

Key Features:
    - Automatic test directory isolation via PYTEST_BASE_TEMP
    - Clean test output with minimal verbosity
    - Proper cleanup of temporary resources
    - Full pytest compatibility - all pytest arguments are passed through
    - Parallel test execution with pytest-xdist

Example Usage:
    # Run all tests in tests directory
    autodebugger run-pytest

    # Run specific test file with pytest arguments
    autodebugger run-pytest tests/test_specific.py -v

    # Run tests matching a pattern with pytest-style arguments
    autodebugger run-pytest tests -k "test_pattern" -v

    # Run tests with coverage
    autodebugger run-pytest tests --cov=src

    # Run tests sequentially
    autodebugger run-pytest tests -n 0

    # Run tests with specific number of workers
    autodebugger run-pytest tests -n 4

Environment Variables:
    PYTEST_BASE_TEMP: Base directory for test temporary directories
                     Default: /tmp
"""
import argparse
import os
import sys
from pathlib import Path
from typing import List, Tuple

import pytest
from autodebugger.autodebugger_logger import logger

def split_pytest_args(args: List[str]) -> Tuple[List[str], List[str]]:
    """Split arguments into autodebugger args and pytest args.
    
    Known pytest arguments:
        -v, --verbose: Increase verbosity
        -q, --quiet: Decrease verbosity
        -k: Only run tests matching the given substring expression
        --pdb: Start the interactive Python debugger on errors
        --cov: Measure coverage
        -x, --exitfirst: Exit instantly on first error
        -s: Shortcut for --capture=no
        -n, --numprocesses: Number of worker processes for parallel testing
        
    Args:
        args: List of command line arguments
        
    Returns:
        Tuple of (autodebugger_args, pytest_args)
    """
    autodebugger_args = []
    pytest_args = []
    
    i = 0
    while i < len(args):
        arg = args[i]
        
        # Check if this is a value for a previous flag (like -k "test_pattern")
        if i > 0 and args[i-1].startswith('-'):
            pytest_args.append(arg)
            i += 1
            continue
            
        # Check for test paths (either exists or looks like a test path)
        # Handle pytest's :: syntax for specific test functions
        base_path = arg.split('::')[0] if '::' in arg else arg
        if os.path.exists(base_path) or (
            not arg.startswith('-') and  # Not an option
            ('tests/' in arg or arg.endswith('.py'))  # Looks like a test path
        ):
            autodebugger_args.append(arg)
            i += 1
            continue
        # All other args go to pytest
            
        pytest_args.append(arg)
        i += 1
        
    return autodebugger_args, pytest_args


def create_parser():
    """Create argument parser for autodebugger CLI."""
    parser = argparse.ArgumentParser(description="Enhanced pytest runner with test isolation")
    subparsers = parser.add_subparsers(dest="command")
    
    # run-pytest command
    run_pytest_parser = subparsers.add_parser("run-pytest",
                                           help="Run tests with pytest")
    run_pytest_parser.add_argument("test_paths", nargs="*",
                                help="Paths to test files or directories")
    
    return parser


def print_help():
    """Print help message."""
    print("""autodebugger - Enhanced pytest runner with test isolation

Usage:
    autodebugger [command] [options] [pytest args]
    
Commands:
    run-pytest  Run tests with pytest
    
Examples:
    autodebugger tests/test_*.py -v              Run tests with pytest output
    autodebugger -v -k test_slow tests/          Run tests matching 'test_slow'
    autodebugger run-pytest tests/ -n 4          Run tests with 4 workers
    autodebugger run-pytest tests/ -n 0          Run tests sequentially

Flags:
    -n N, --numprocesses=N  Number of worker processes (N=0 for sequential, N=auto for automatic)
    -h, --help             Show this help message
    
All pytest arguments (like -v, -k, etc.) are supported.""")


def run_pytest(args):
    """Run tests with pytest."""
    try:
        # Split remaining args into autodebugger and pytest args
        autodebugger_args, pytest_args = split_pytest_args(args)
        
        # Parse autodebugger args
        parser = create_parser()
        args = parser.parse_args(["run-pytest"] + autodebugger_args)
        
        # If no test paths provided or only default "tests", find all tests/ directories
        if not args.test_paths or (len(args.test_paths) == 1 and args.test_paths[0] == "tests"):
            cwd = Path.cwd()
            test_paths = []

            # Check root level tests/
            if (cwd / "tests").is_dir():
                test_paths.append("tests")
                
            # Check one level down
            for item in cwd.iterdir():
                if item.is_dir() and not item.name.startswith('.'):
                    tests_dir = item / "tests"
                    if tests_dir.is_dir():
                        rel_path = str(item.name / Path("tests"))
                        test_paths.append(rel_path)
            
            print(f"autodebugger: found test paths: {test_paths}")
            if not test_paths:
                print("autodebugger: error - No test directories found")
                sys.exit(1)
        else:
            test_paths = args.test_paths
            
        # Check if test paths exist, accounting for pytest's :: syntax
        valid_paths = False
        for p in test_paths:
            base_path = p.split('::')[0] if '::' in p else p
            if os.path.exists(base_path):
                valid_paths = True
                break
                
        if not valid_paths:
            print("autodebugger: error - No test paths found")
            sys.exit(1)
            
        # First collect the tests to get the count by running pytest in collection mode
        import subprocess
        result = subprocess.run(
            [sys.executable, "-m", "pytest", "--collect-only", "-q"] + test_paths,
            capture_output=True,
            text=True
        )
        # Parse output like "tests/test_autodebugger.py: 2"
        num_tests = 0
        for line in result.stdout.splitlines():
            if ".py: " in line and not line.startswith("platform"):
                num_tests += int(line.split(": ")[1])
            
        # Add default parallel execution if not specified
        if not any(arg.startswith('-n') or arg.startswith('--numprocesses') for arg in pytest_args):
            pytest_args.extend(['-n', 'auto', '-q', '--tb=short'])  # Default to parallel with clean output
            
            # Add worksteal distribution for better handling of test duration differences
            if not any(arg.startswith('--dist') for arg in pytest_args):
                pytest_args.extend(['--dist=worksteal', '--maxprocesses', '0'])
            
            # Set maxprocesses based on test count
            cpu_count = os.cpu_count() or 4  # Default to 4 if cpu_count() returns None
            max_workers = min(num_tests, max(cpu_count // 2, 1))
            if not any(arg.startswith('--maxprocesses') for arg in pytest_args):
                pytest_args.extend(['--maxprocesses', str(max_workers)])
            
            # Add loadfile distribution for coverage
            if any(arg.startswith('--cov') for arg in pytest_args):
                # Override dist mode for coverage to avoid conflicts
                pytest_args = [arg for arg in pytest_args if not arg.startswith('--dist')]
                pytest_args.extend(['--dist=loadfile'])
        
        # Run tests with pytest
        all_pytest_args = test_paths + pytest_args
        print(f"autodebugger: running tests with pytest args - {' '.join(all_pytest_args)}")
        result = pytest.main(all_pytest_args)
        
        # Print test logs
        logger.print_test_logs()
        
        # Run post-test verifications
        from autodebugger.testutil import run_post_test_verifications
        if not run_post_test_verifications(logger, result):
            sys.exit(1)
            
        sys.exit(result)
        
    except KeyboardInterrupt:
        print("\nautodebugger: interrupted by user")
        sys.exit(1)


def main() -> int:
    """Main entry point for autodebugger CLI."""
    if sys.argv[1:] and (sys.argv[1] == "--help" or sys.argv[1] == "-h"):
        print_help()
        return 0
        
    if sys.argv[1:] and sys.argv[1] == "run-pytest":
        run_pytest(sys.argv[2:])
    else:
        run_pytest(sys.argv[1:])
        
    return 0


if __name__ == "__main__":
    main()
