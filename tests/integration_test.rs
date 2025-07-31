use autodebugger::AutoDebugger;

#[test]
fn test_basic_commands() -> anyhow::Result<()> {
    let debugger = AutoDebugger::new();
    
    // Test echo command
    let result = debugger.run_command("echo 'Hello, World!'")?;
    assert!(result.success);
    assert_eq!(result.stdout.trim(), "Hello, World!");
    
    // Test pwd command
    let result = debugger.run_command("pwd")?;
    assert!(result.success);
    assert!(!result.stdout.is_empty());
    
    // Test ls command
    let result = debugger.run_command("ls")?;
    assert!(result.success);
    
    Ok(())
}