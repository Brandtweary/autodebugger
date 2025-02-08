# autodebugger

A debugging tool that augments test output for LLM debugging workflows. autodebugger performs code analysis after test runs to provide helpful context to users and LLM agents. Currently compatible with Python code and pytest, with potential for additional language support in the future.

## Features

- **Enhanced Test Output**: Provides rich context about test failures and code state
- **Test Directory Isolation**: Each test gets its own unique temporary directory
- **Full Pytest Compatibility**: All pytest arguments and plugins are supported
- **Parallel Execution**: Built-in parallel test execution using pytest-xdist

## Installation

The autodebugger can be installed directly from the source directory.

## Usage

### Basic Usage

Run all tests in all `tests/` directories (root and one level down):
```bash
autodebugger
# or with verbosity
autodebugger -v
```

Run all tests in a specific directory:
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

## Example Test

Here's an example showing how to use autodebugger in your tests:

```python
import pytest
from pathlib import Path
from autodebugger.testutil import generate_test_dir

class TestFileOperations:
    """Test suite for file operations."""
    
    @pytest.fixture(autouse=True)
    def setup_test_dir(self, request, tmp_path):
        """Set up test directory with meaningful name."""
        # Generate a unique test directory path using the test name as prefix
        test_name = request.node.name.replace("[", "").replace("]", "")
        self.test_dir = generate_test_dir(prefix=test_name)
        
        # tmp_path fixture creates the directory and handles cleanup
        self.file_path = self.test_dir / "test.txt"
        yield
        # Cleanup handled automatically by pytest
    
    def test_write_and_read(self):
        """Test basic file write and read operations."""
        with open(self.file_path, "w") as f:
            f.write("test data")
            
        with open(self.file_path, "r") as f:
            assert f.read() == "test data"
```

## Contributing

When contributing to autodebugger:

1. Follow the naming convention: `test_*.py` for test files
2. Use descriptive test names that reflect what is being tested
3. Use `generate_test_dir()` to create uniquely named test directories
4. Add proper type hints and docstrings
5. Tests are run in parallel by default - ensure your tests are isolated and don't interfere with each other

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
