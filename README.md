# Autodebugger

Developer utilities designed to enhance LLM-assisted coding workflows. Autodebugger provides essential tools that help language models work more effectively with real-world codebases, including automated log verbosity detection, debug code removal, and rotating file logging.

## Installation

### Configuration Setup

Before using autodebugger, set up your configuration:

```bash
# Copy the example configuration
cp config.example.yaml config.yaml

# Edit config.yaml to customize thresholds (optional)
```

### As a Git Submodule

If autodebugger is included as a submodule in another project:

```bash
# Clone the parent repository with submodules
git clone --recursive <parent-repo-url>

# Or if already cloned, initialize submodules
git submodule update --init --recursive

# Navigate to the autodebugger directory
cd path/to/autodebugger

# Install the CLI tool
cargo install --path .
```

### As a Standalone Repository

```bash
# Clone the repository
git clone <autodebugger-repo-url>
cd autodebugger

# Install the CLI tool
cargo install --path .
```

After installation, the `autodebugger` command will be available in your terminal.

## CLI Commands

### remove-debug
Remove all `debug!` macro calls from Rust source files. Follows the convention that debug! calls are temporary and should be removed before committing.

```bash
# Remove debug! calls from default paths (configured in config.yaml)
autodebugger remove-debug

# Remove from multiple paths
autodebugger remove-debug src/ tests/ examples/

# Remove from specific file or directory
autodebugger remove-debug src/main.rs
autodebugger remove-debug /path/to/project

# Preview what would be removed without making changes
autodebugger remove-debug --dry-run

# Verbose mode shows which files are processed and warnings
autodebugger remove-debug --verbose
```

The command conservatively removes only standalone `debug!` calls on their own lines. It skips:
- Multiline debug! macros
- debug! calls mixed with other code on the same line
- debug! in comments (might be examples or documentation)
- debug! in match arms or lambdas

### Legacy Command Execution

```bash
# Run any shell command through autodebugger
autodebugger run ls
autodebugger run pwd
autodebugger run "echo 'Hello, World!'"
autodebugger run "cargo build"
```

## Library Usage

```rust
use autodebugger::Autodebugger;

let debugger = Autodebugger::new();
let result = debugger.run_command("echo 'Hello, World!'")?;
println!("Output: {}", result.stdout);
```

## Features

### Complete Tracing Subscriber
A production-ready tracing subscriber optimized for LLM-assisted development. The `init_logging()` function provides a complete setup with sensible defaults:

```rust
use autodebugger::init_logging;

fn main() {
    // Initialize with all features enabled
    let verbosity_layer = init_logging(None);  // Uses "info" as default
    
    // Your application code here...
    
    // Check for excessive verbosity at shutdown
    if let Some(report) = verbosity_layer.check_and_report() {
        tracing::warn!("{}", report);
    }
}
```

**Key features of the tracing subscriber:**
- **Clean console output**: No timestamps or ANSI codes cluttering terminal output
- **Smart level prefixes**: INFO prefix is hidden for cleaner default output, while DEBUG/WARN/ERROR are shown
- **Conditional location info**: File:line information only shown for ERROR and WARN levels where it's most useful
- **Sled/pagecache filtering**: Automatically suppresses verbose output from sled database (common in Rust apps)
- **Verbosity monitoring**: Built-in detection of excessive logging with configurable thresholds
- **Environment-aware**: Respects RUST_LOG environment variable settings

### Rotating File Logger

Automatic file logging that works like the `tee` command - outputs to both console and rotating log files. Perfect for applications that need persistent logs with automatic cleanup:

```rust
use autodebugger::{init_logging_with_file, RotatingFileConfig};

fn main() {
    // Configure rotating file logger
    let file_config = RotatingFileConfig {
        log_directory: std::path::PathBuf::from("logs"),
        filename: "myapp.log".to_string(),
        max_files: 10,        // Keep 10 rotating files
        max_size_mb: 10,      // Rotate at 10MB
        console_output: true, // Also output to console
    };
    
    // Initialize with both console and file output
    let (verbosity_layer, _file_guard) = init_logging_with_file(
        Some("info"), 
        Some(file_config)
    ).expect("Failed to initialize logging");
    
    // All tracing output now goes to both console AND files
    tracing::info!("This appears in console and logs/myapp.log");
    
    // Keep _file_guard alive for the duration of logging
}
```

**Configuration via config file:**

```yaml
# autodebugger config.yaml
log_rotation_count: 5  # Keep 5 log files (default: 10)
```

```rust
// Load rotation count from config
let config = autodebugger::Config::load().unwrap_or_default();
let file_config = RotatingFileConfig {
    max_files: config.log_rotation_count,  // Uses config value
    // ... other settings
};
```

### Log Verbosity Detection
Automatically detect when your application is generating excessive logs. The `VerbosityCheckLayer` integrates seamlessly with the `tracing` ecosystem to monitor log output and warn when thresholds are exceeded.

### Command Execution
- Structured command execution with clean result types
- Working directory management
- Sequential command chains with early exit on failure
- Input piping support
- Async command execution

### Custom Tracing Components

If you need more control, you can use the individual components:

Add to your `Cargo.toml`:
```toml
[dependencies]
autodebugger = "0.1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["registry", "fmt", "env-filter"] }
```

Build your own custom tracing setup:
```rust
use autodebugger::{VerbosityCheckLayer, ConditionalLocationFormatter, create_base_env_filter};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    // Create individual components
    let env_filter = create_base_env_filter("info");  // With sled suppression
    let verbosity_layer = VerbosityCheckLayer::new();
    
    // Build custom subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer()
            .event_format(ConditionalLocationFormatter))  // Clean formatting
        .with(verbosity_layer.clone())
        .init();
    
    // Your application code here...
    
    // Check verbosity at shutdown
    if let Some(report) = verbosity_layer.check_and_report() {
        tracing::warn!("{}", report);
    }
}
```

The checker automatically detects your configured log level and applies thresholds from your config.yaml. Default thresholds:
- `INFO`: 50 total logs
- `DEBUG`: 100 total logs  
- `TRACE`: 200 total logs

You can customize these thresholds by editing config.yaml:
```yaml
verbosity:
  info_threshold: 75    # Allow more logs in production
  debug_threshold: 150  # Adjust for your debugging needs
```

When exceeded, you'll get a clear breakdown showing which log levels are contributing to the verbosity.

## Examples

Run the examples to see the autodebugger in action:

```bash
# Basic usage example
cargo run --example basic_usage

# Interactive debugging (simulates LLM agent usage)
cargo run --example interactive_debugging
```

## Testing

```bash
cargo test
```

## Sample Project

The `sample_project/` directory contains a simple Rust project that can be used for testing the autodebugger's capabilities.

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