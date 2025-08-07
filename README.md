# Autodebugger

An integrated code editor and development tool designed for use by LLM agents. This Rust crate provides programmatic code manipulation features that enable LLMs to debug and modify code more effectively than traditional approaches.

## Installation

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

## CLI Usage

```bash
# Run any shell command through autodebugger
autodebugger ls
autodebugger pwd
autodebugger echo "Hello, World!"
autodebugger cargo build
```

## Library Usage

```rust
use autodebugger::Autodebugger;

let debugger = Autodebugger::new();
let result = debugger.run_command("echo 'Hello, World!'")?;
println!("Output: {}", result.stdout);
```

## Features

- Simple command execution with structured results
- Working directory management
- Sequential command execution with early exit on failure
- Input piping support
- Async command execution support

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