use autodebugger::AutoDebugger;
use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    println!("=== AutoDebugger Interactive Example ===");
    println!("This simulates how an LLM agent might use the autodebugger.");
    println!("Type 'exit' to quit.\n");

    let debugger = AutoDebugger::new();

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let command = input.trim();

        if command == "exit" {
            break;
        }

        if command.is_empty() {
            continue;
        }

        match debugger.run_command(command) {
            Ok(result) => {
                if !result.stdout.is_empty() {
                    println!("Output:\n{}", result.stdout);
                }
                if !result.stderr.is_empty() {
                    println!("Error:\n{}", result.stderr);
                }
                if !result.success {
                    println!("Command failed with exit code: {}", result.exit_code);
                }
            }
            Err(e) => {
                println!("Failed to execute command: {}", e);
            }
        }
    }

    println!("Goodbye!");
    Ok(())
}