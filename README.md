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
```

Run all tests in a specific directory:
```bash
autodebugger tests/
```

Run a specific test file:
```bash
autodebugger tests/test_specific.py
```

Run tests with increased verbosity:
```bash
autodebugger -v
```

Run tests with INFO logs shown:
```bash
autodebugger -i
```

### Advanced Usage

Run tests with no capture (show all log levels):
```bash
autodebugger -s
```

Run tests matching a pattern:
```bash
autodebugger -k "test_pattern" tests/
```

Run tests with coverage reporting:
```bash
autodebugger --cov=src
```

Run a specific subcommand:
```bash
autodebugger run-pytest  # run-pytest is the default subcommand
```

### Parallel Execution

By default, tests run in parallel using pytest-xdist with half of your CPU cores. You can control this behavior:

Run tests sequentially:
```bash
autodebugger -n 0
```

Run tests with a specific number of workers:
```bash
autodebugger -n 4  # Use 4 workers
```

Use automatic worker count (default behavior):
```bash
autodebugger -n auto  # Uses CPU_COUNT/2 workers
```

#### Fixtures in Parallel Mode

pytest-xdist handles fixtures seamlessly in parallel execution:
- Each worker process gets its own isolated fixture tree
- Function and class-scoped fixtures are created fresh for each test
- Session-scoped fixtures are created once per worker
- Temporary directories (via `tmp_path` and `generate_test_dir`) are always unique and isolated

This means you can freely use fixtures in your tests without worrying about parallel execution conflicts.

### Environment Variables

- `PYTEST_BASE_TEMP`: Base directory for test temporary directories (default: `/tmp`)

## Logging System

The autodebugger provides a logging system that integrates with pytest's output capture. This allows you to:
- Log messages at different severity levels (DEBUG, INFO, WARNING, etc.)
- Automatically show all logs for failed tests
- Control log visibility based on flags

The logger has three visibility modes:
- Default: Only WARNING and above are shown
- Info (`-i`): INFO and above are shown
- No Capture (`-s`): All log levels are shown

For failed tests, all log messages are shown regardless of mode.

The `-v` flag increases pytest's output verbosity but does not affect the logger's behavior.

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

## Test Configuration and conftest.py

When running tests with autodebugger, it requires certain pytest fixtures and hooks to be present in order to capture test logs. To handle this, autodebugger will automatically create a `conftest.py` file in your working directory (where you run the `autodebugger` command from).

### Existing conftest.py Files

If you already have a `conftest.py` file in your project, you have two options:

1. **Remove your conftest.py**: Let autodebugger manage the test configuration. This is the simplest approach if you don't have custom fixtures or hooks.

2. **Import autodebugger's hooks**: If you need to keep your own `conftest.py`, simply import autodebugger's hooks:

```python
# Import autodebugger's hooks and fixtures
from autodebugger.conftest import *
```

That's it! The hooks will be automatically registered with pytest.

## Example Test

Here's an example showing how to use autodebugger in your tests:

```python
import pytest
from pathlib import Path
from autodebugger.testutil import generate_test_dir
from autodebugger.autodebugger_logger import logger

class TestFileOperations:
    """Test suite for file operations."""
    
    @pytest.fixture(autouse=True)
    def setup_test_dir(self, request, tmp_path):
        """Set up test directory with meaningful name."""
        # Set up logging for this test
        logger.set_current_request(request)
        logger.debug("Setting up test directory")
        
        # Generate a unique test directory path using the test name as prefix
        test_name = request.node.name.replace("[", "").replace("]", "")
        self.test_dir = generate_test_dir(prefix=test_name)
        logger.info(f"Created test directory: {self.test_dir}")
        
        # tmp_path fixture creates the directory and handles cleanup
        self.file_path = self.test_dir / "test.txt"
        logger.debug(f"Test file path: {self.file_path}")
        yield
        
        # Cleanup handled automatically by pytest
        logger.debug("Test directory cleanup starting")
    
    def test_write_and_read(self, request):
        """Test basic file write and read operations."""
        logger.set_current_request(request)
        
        # Write test data
        test_data = "test data"
        logger.info(f"Writing data to {self.file_path}")
        try:
            with open(self.file_path, "w") as f:
                f.write(test_data)
        except IOError as e:
            logger.error(f"Failed to write to file: {e}")
            raise
            
        # Read and verify
        logger.debug("Reading back written data")
        try:
            with open(self.file_path, "r") as f:
                content = f.read()
                if content != test_data:
                    logger.warning(f"Content mismatch - Expected: {test_data}, Got: {content}")
                assert content == test_data
        except IOError as e:
            logger.error(f"Failed to read from file: {e}")
            raise
        
        logger.info("File operations completed successfully")
```

## Contributing

When contributing to autodebugger:

1. Follow the naming convention: `test_*.py` for test files
2. Use descriptive test names that reflect what is being tested
3. Use `generate_test_dir()` to create uniquely named test directories
4. Add proper type hints and docstrings
5. Tests are run in parallel by default - ensure your tests don't have external dependencies or side effects that could interfere with parallel execution

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
