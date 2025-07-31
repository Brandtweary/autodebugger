use autodebugger::AutoDebugger;

fn main() -> anyhow::Result<()> {
    let debugger = AutoDebugger::new();
    
    println!("=== AutoDebugger Basic Verification ===\n");
    
    // Test 1: echo
    println!("1. Testing echo:");
    let result = debugger.run_command("echo 'Hello from AutoDebugger!'")?;
    if result.success {
        println!("   ✓ Output: {}", result.stdout.trim());
    } else {
        println!("   ✗ Failed: {}", result.stderr);
        return Ok(());
    }
    
    // Test 2: pwd
    println!("\n2. Testing pwd:");
    let result = debugger.run_command("pwd")?;
    if result.success {
        println!("   ✓ Working directory: {}", result.stdout.trim());
    } else {
        println!("   ✗ Failed: {}", result.stderr);
        return Ok(());
    }
    
    // Test 3: ls
    println!("\n3. Testing ls:");
    let result = debugger.run_command("ls")?;
    if result.success {
        let files: Vec<&str> = result.stdout.trim().split('\n').collect();
        println!("   ✓ Found {} files/directories", files.len());
        for file in files.iter().take(3) {
            println!("     - {}", file);
        }
        if files.len() > 3 {
            println!("     ... and {} more", files.len() - 3);
        }
    } else {
        println!("   ✗ Failed: {}", result.stderr);
        return Ok(());
    }
    
    println!("\n✅ All basic commands working correctly!");
    
    Ok(())
}