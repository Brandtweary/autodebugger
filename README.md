# Autodebugger

A pytest-based test runner that provides enhanced test isolation and directory management.

## Features

- **Test Directory Isolation**: Each test gets its own unique temporary directory via `PYTEST_BASE_TEMP`
- **Clean Output**: Minimal verbosity, focusing on what matters
- **Automatic Cleanup**: Temporary resources are cleaned up after each test
- **Full Pytest Compatibility**: All pytest arguments and plugins are supported
- **Parallel Execution**: Built-in parallel test execution using pytest-xdist

## Installation

The autodebugger can be installed directly from the source directory.

## Usage

### Basic Usage

Run all tests in the `tests` directory:
```bash
autodebugger tests/
# or with explicit subcommand
autodebugger run-pytest tests/
```

Run a specific test file:
```bash
autodebugger tests/test_specific.py
```

### Advanced Usage

Run tests with increased verbosity:
```bash
autodebugger tests -v
```

Run tests matching a pattern:
```bash
autodebugger -k "test_pattern" tests/
```

### Parallel Execution

By default, tests run in parallel using pytest-xdist with half of your CPU cores. You can control this behavior:

Run tests sequentially:
```bash
autodebugger tests/ -n 0
```

Run tests with a specific number of workers:
```bash
autodebugger tests/ -n 4  # Use 4 workers
```

Use automatic worker count (default behavior):
```bash
autodebugger tests/ -n auto  # Uses CPU_COUNT/2 workers
```

### Coverage

Run tests with coverage reporting:
```bash
autodebugger tests --cov=src
```

Coverage works seamlessly with parallel execution. When using `--cov`, tests are automatically distributed by file to avoid coverage conflicts.

### Environment Variables

- `PYTEST_BASE_TEMP`: Base directory for test temporary directories (default: `/tmp`)

## Integration with pytest

The autodebugger is fully compatible with pytest. All pytest arguments are passed through, including:

- `-v`, `--verbose`: Increase verbosity
- `-q`, `--quiet`: Decrease verbosity
- `-k PATTERN`: Only run tests matching the pattern
- `--pdb`: Start debugger on failures
- `--cov`: Enable coverage reporting
- `-n`, `--numprocesses`: Control parallel execution

For a complete list of supported arguments, run:
```bash
autodebugger --help
```

## Best Practices

1. **Test Directory Management**
   - Use `generate_test_dir()` from `testutil.py` to create test directories
   - Access the base temp directory via `PYTEST_BASE_TEMP` environment variable

2. **Resource Cleanup**
   - Always use context managers (`with` statements) for file operations
   - Use `try`/`finally` blocks to ensure cleanup runs even if tests fail

Example:
```python
from pathlib import Path
from autodebugger.testutil import generate_test_dir

def test_file_operations():
    # Create unique test directory
    test_dir = generate_test_dir()
    file_path = test_dir / "test.txt"
    
    try:
        # Perform test operations
        with open(file_path, "w") as f:
            f.write("test data")
            
        # Verify results
        with open(file_path, "r") as f:
            assert f.read() == "test data"
    finally:
        # Clean up is handled automatically by autodebugger
        pass
```

## Contributing

When adding new tests:

1. Follow the naming convention: `test_*.py` for test files
2. Use descriptive test names that reflect what is being tested
3. Keep tests focused and isolated
4. Clean up all temporary resources
5. Add proper type hints and docstrings

## Development

The autodebugger is designed to be extensible. It uses pytest's collection mechanism
while providing its own execution and reporting infrastructure. This allows for features
like parallel execution while maintaining compatibility with pytest's ecosystem.
