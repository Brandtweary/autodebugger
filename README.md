# Autodebugger

Developer utilities designed to enhance LLM-assisted coding workflows. Autodebugger provides essential tools that help language models work more effectively with real-world codebases, addressing common pain points like excessive logging and complex command orchestration.

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

### Log Verbosity Detection
Automatically detect when your application is generating excessive logs. The `VerbosityCheckLayer` integrates seamlessly with the `tracing` ecosystem to monitor log output and warn when thresholds are exceeded.

### Command Execution
- Structured command execution with clean result types
- Working directory management
- Sequential command chains with early exit on failure
- Input piping support
- Async command execution

### Log Verbosity Checker Usage

Add to your `Cargo.toml`:
```toml
[dependencies]
autodebugger = "0.1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["registry"] }
```

Integrate with your existing tracing setup:
```rust
use autodebugger::VerbosityCheckLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    // Create the verbosity checking layer
    let verbosity_layer = VerbosityCheckLayer::new();
    
    // Add it to your tracing subscriber
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
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