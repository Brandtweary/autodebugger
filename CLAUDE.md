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
- **tests/**: Integration tests
  - **integration_test.rs**: Core integration tests
  - **rotating_logger_test.rs**: Rotating logger tests
- **autodebugger_logs/**: Generated log files (git-ignored)
- **config.yaml**: Runtime configuration (not tracked)
- **config.example.yaml**: Configuration template with all options documented
- **Cargo.toml**: Dependencies and metadata
- **Cargo.lock**: Locked dependency versions
- **README.md**: User documentation and installation guide

## Development Guidelines
- **IMPORTANT**: After making changes, run `cargo install --path .` to update the global autodebugger command
- Logging: Use `tracing` macros - `info!()`, `warn!()`, `error!()`, `debug!()`, `trace!()`
- Error handling: Use `anyhow::Result` with context