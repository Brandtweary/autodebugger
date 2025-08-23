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
    
    // Check that a timestamped log file was created in the directory
    let test_path = Path::new(&test_dir);
    assert!(test_path.exists(), "Test directory should exist");
    
    // Find the timestamped log file
    let entries = fs::read_dir(&test_path)
        .expect("Should be able to read test directory");
    
    let log_files: Vec<_> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().and_then(|s| s.to_str()) == Some("log")
        })
        .collect();
    
    assert!(!log_files.is_empty(), "At least one log file should exist");
    
    // Read the first log file we find
    let log_file = &log_files[0];
    let contents = fs::read_to_string(log_file.path())
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