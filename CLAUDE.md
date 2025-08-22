# AUTODEBUGGER DEVELOPMENT GUIDE

## Build/Test Commands
```bash
# In autodebugger root
cargo check                      # Quick syntax check
cargo build                      # Build autodebugger
cargo test                       # Run full test suite
cargo install --path .           # Install autodebugger CLI globally
autodebugger --help              # View all available commands
```

## CLI Commands

### Documentation & Code Quality
- `validate-docs [PATHS]`: Validate module documentation (//! format only)
  - `--verbose`: Show all files including skipped simple modules
  - `--strict`: Treat warnings as errors (exit code 1)
- `remove-debug [PATHS]`: Remove debug! macro calls from Rust files
  - `--dry-run`: Preview changes without modifying files
  - `--verbose`: Show detailed processing information

### Worktree Monitoring
- `monitor <PATH>`: Monitor git worktrees for changes
  - `--format [json|text]`: Output format
- `diff [WORKTREE]`: Show diffs across worktrees
  - `--summary`: Show summary only
  - `--path <PATH>`: Workspace path
- `status`: Show status of all worktrees
  - `--path <PATH>`: Workspace path
  - `--json`: Output as JSON
- `context [TYPE]`: Get aggregated context (local-tasks, status)
  - `--path <PATH>`: Workspace path

### Legacy
- `run <COMMAND>`: Execute shell command through autodebugger

## Project Structure

### Core Directories
- **src/**: Main source code
  - **main.rs**: CLI entry point with all command handlers
  - **lib.rs**: Core library exports and command execution
  - **config.rs**: YAML configuration management
  - **validate_docs.rs**: Documentation validation with configurable thresholds
  - **remove_debug.rs**: Debug macro removal with multi-line support
  - **rotating_file_logger.rs**: Size-based log rotation with numbered backups
  - **tracing_subscriber.rs**: Advanced tracing with verbosity detection
  - **monitor/**: Worktree monitoring
    - **mod.rs**: Monitor orchestration
    - **worktree.rs**: Git worktree detection
    - **diff.rs**: Diff tracking and aggregation
- **examples/**: Usage examples
  - **basic_usage.rs**: Simple command execution
  - **interactive_debugging.rs**: LLM agent simulation
  - **rotating_logger.rs**: Log rotation demonstration
- **tests/**: Integration tests
- **sample_project/**: Test Rust project for debugging
- **autodebugger_logs/**: Generated log files (git-ignored)

### Configuration Files
- **config.yaml**: Runtime configuration (not tracked)
- **config.example.yaml**: Configuration template with all options documented
- **Cargo.toml**: Dependencies and metadata
- **README.md**: User documentation and installation guide

## Development Guidelines
- Logging: Use `tracing` macros - `info!()`, `warn!()`, `error!()`, `debug!()`, `trace!()`
- Error handling: Use `anyhow::Result` with context