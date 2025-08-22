//! Example of using the rotating file logger
//! 
//! Run with: cargo run --example rotating_logger

use autodebugger::RotatingFileLogger;
use tracing::{info, warn, error, debug};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize rotating file logger with custom configuration
    let _guard = RotatingFileLogger::init()
        .with_directory("example_logs")  // Custom log directory
        .with_filename("example.log")    // Custom filename
        .with_max_files(5)                // Keep only 5 rotating files
        .with_max_size_mb(1)              // Rotate at 1MB (small for demo)
        .with_console(true)               // Also output to console
        .build()?;

    // Now all tracing output goes to both console AND rotating files!
    info!("Starting rotating logger example");
    warn!("This is a warning message");
    error!("This is an error message (not a real error!)");
    debug!("Debug information here");

    // Generate some logs to demonstrate rotation
    info!("Generating logs to demonstrate rotation...");
    
    for i in 0..100 {
        info!("Log entry {}: The quick brown fox jumps over the lazy dog", i);
        debug!("Additional debug info for entry {}", i);
        
        if i % 10 == 0 {
            warn!("Periodic warning at entry {}", i);
        }
        
        // Small delay to make it feel more realistic
        thread::sleep(Duration::from_millis(10));
    }

    info!("Example completed! Check the 'example_logs' directory for log files");
    info!("You should see:");
    info!("  - example.log (current log)");
    info!("  - example.log.1 (previous rotation)");
    info!("  - example.log.2 (if we generated enough logs)");
    
    Ok(())
}