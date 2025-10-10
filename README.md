# Autodebugger

Developer utilities for LLM-assisted coding: documentation validation, debug removal, and intelligent logging.

## Installation

```bash
# Standalone
git clone <autodebugger-repo-url> && cd autodebugger
cargo install --path .

# As submodule  
git submodule update --init --recursive
cd path/to/autodebugger && cargo install --path .

# Configure (optional - sensible defaults provided)
cp config.example.yaml config.yaml
```

## CLI Commands

```bash
# Documentation validation (requires //! format)
autodebugger validate-docs [PATHS...]    # Default: paths from config
  --verbose, -v                          # Show all files checked
  --strict, -s                           # Treat warnings as errors

# Debug removal
autodebugger remove-debug [PATHS...]     # Default: paths from config
  --dry-run, -d                          # Preview changes without modifying
  --verbose, -v                          # Show detailed output

# Worktree operations
autodebugger monitor <PATH>              # Monitor worktrees for changes
  --format, -f [json|text]               # Output format (default: text)

autodebugger diff [WORKTREE]            # Show diffs across worktrees
  --summary, -s                          # Show summary only

autodebugger status                     # Show status of all worktrees
  --json, -j                             # Output as JSON

autodebugger context [TYPE] [--path PATH] # Get aggregated context
  TYPE: local-tasks|status (default: status)

# Legacy
autodebugger run <COMMAND>              # Run a command (legacy mode)
```

## Library Usage

```rust
use autodebugger::{Autodebugger, init_logging, init_logging_with_file, FileLogConfig};

// Command execution
let debugger = Autodebugger::new();
let result = debugger.run_command("cargo build")?;

// Initialize tracing (console only)
let verbosity_layer = init_logging(None, None, None);  // Uses "info", stdout defaults

// Initialize with file logging (dual output)
let file_config = FileLogConfig {
    file_path: "logs/app.log".to_string(),
    truncate: true,
};
let verbosity_layer = init_logging_with_file(
    Some("info"),
    None,
    Some("stderr"),  // Console to stderr, file gets both
    file_config
);

// Check verbosity at shutdown (optional)
if let Some(report) = verbosity_layer.check_and_report() {
    tracing::warn!("{}", report);
}
```

## Key Features

**Tracing Subscriber**: Clean console output, smart verbosity detection
- `init_logging()` - Quick setup with sensible defaults (console only)
- `init_logging_with_file()` - Dual console + file output
- `VerbosityCheckLayer` - Detects excessive logging patterns
- `ConditionalLocationFormatter` - Shows file:line only for WARN/ERROR
- `FileLogConfig` - Configure file logging behavior

**File Logging**: Optional dual output to both console and file
- Thread-safe file writing with Arc<Mutex<File>>
- Configurable truncate vs append mode
- No ANSI colors in log files
- Automatic directory creation
- Graceful fallback to console-only on file errors

**Verbosity Detection**: Configurable thresholds warn when logs exceed limits
```yaml
verbosity:
  info_threshold: 50    # Max INFO logs before warning
  debug_threshold: 100  # Max DEBUG logs
  trace_threshold: 200  # Max TRACE logs
```

## Configuration

All settings in `config.yaml` (see `config.example.yaml` for options):
- `validate_docs`: Documentation validation thresholds
- `remove_debug`: Default paths for debug removal  
- `verbosity`: Log verbosity thresholds

## Testing

```bash
cargo test
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.