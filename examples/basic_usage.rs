use autodebugger::AutoDebugger;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    println!("=== AutoDebugger Basic Usage Example ===\n");

    let mut debugger = AutoDebugger::new();

    println!("1. Running a simple command:");
    let result = debugger.run_command("echo 'Hello from AutoDebugger!'")?;
    println!("   Output: {}", result.stdout);
    println!("   Success: {}\n", result.success);

    println!("2. Listing files in current directory:");
    let result = debugger.run_command("ls -la")?;
    println!("   Files:\n{}", result.stdout);

    println!("3. Running multiple commands sequentially:");
    let commands = vec![
        "echo 'First command'",
        "echo 'Second command'",
        "echo 'Third command'",
    ];
    let results = debugger.run_commands_sequential(commands)?;
    for (i, result) in results.iter().enumerate() {
        println!("   Command {}: {}", i + 1, result.stdout.trim());
    }

    println!("\n4. Working with the sample project:");
    let sample_project_path = PathBuf::from("sample_project");
    if sample_project_path.exists() {
        debugger.set_working_dir(sample_project_path)?;
        
        println!("   Building the sample project:");
        let result = debugger.run_command("cargo build")?;
        if result.success {
            println!("   ✓ Build successful!");
        } else {
            println!("   ✗ Build failed: {}", result.stderr);
        }

        println!("\n   Running the sample project:");
        let result = debugger.run_command("cargo run")?;
        println!("{}", result.stdout);

        println!("\n   Running tests:");
        let result = debugger.run_command("cargo test")?;
        println!("{}", result.stdout);
    } else {
        println!("   Sample project not found. Make sure to run this from the autodebugger directory.");
    }

    Ok(())
}