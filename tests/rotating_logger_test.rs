//! Test for the rotating file logger

use autodebugger::RotatingFileLogger;
use std::fs;
use std::path::Path;
use tracing::info;

#[test]
fn test_rotating_logger_creates_files() {
    // Use a unique test directory
    let test_dir = format!("test_logs_{}", std::process::id());
    
    // Initialize the logger
    let _guard = RotatingFileLogger::init()
        .with_directory(&test_dir)
        .with_filename("test.log")
        .with_max_files(3)
        .with_max_size_mb(1)
        .with_console(false)  // Disable console for testing
        .build()
        .expect("Failed to initialize rotating logger");
    
    // Generate some log entries
    info!("Test log entry 1");
    info!("Test log entry 2");
    info!("Test log entry 3");
    
    // Check that the log file was created
    let log_path = Path::new(&test_dir).join("test.log");
    assert!(log_path.exists(), "Log file should exist");
    
    // Read the log file
    let contents = fs::read_to_string(&log_path)
        .expect("Should be able to read log file");
    
    // Verify content
    assert!(contents.contains("Test log entry 1"));
    assert!(contents.contains("Test log entry 2"));
    assert!(contents.contains("Test log entry 3"));
    
    // Clean up
    drop(_guard);
    fs::remove_dir_all(&test_dir).ok();
    
    println!("✅ Rotating logger test passed!");
}