//! Rotating file logger with automatic size-based rotation and timestamped archives
//! 
//! This module provides a comprehensive rotating file logging solution that can be integrated
//! into any Rust application. It automatically manages log files, rotating them based on size
//! limits and maintaining a configurable number of historical logs. This is essential for
//! long-running applications that need to manage disk space while preserving log history.
//!
//! ## Features
//!
//! ### Automatic Rotation
//! - Size-based rotation: Automatically rotates when log exceeds configured size
//! - Numbered backups: Maintains logs as app.log, app.log.1, app.log.2, etc.
//! - Configurable retention: Control how many historical logs to keep
//! - Atomic operations: Thread-safe file rotation without data loss
//!
//! ### Flexible Configuration
//! - Custom log directory: Store logs in any location
//! - Configurable filename: Use any base filename for logs
//! - Size limits: Set maximum file size before rotation (in MB)
//! - Retention policy: Specify number of historical files to maintain
//! - Console mirroring: Optionally output to both console and file
//!
//! ### Integration with Tracing
//! - Seamless tracing-subscriber integration
//! - Works with all tracing macros (info!, debug!, trace!, etc.)
//! - Preserves structured logging and spans
//! - Compatible with custom formatters
//!
//! ## Architecture
//!
//! The module consists of three main components:
//!
//! 1. **RotatingFileLogger**: Core rotation logic and file management
//! 2. **RotatingWriterWrapper**: Thread-safe writer implementation for tracing
//! 3. **RotatingFileGuard**: RAII guard that ensures proper cleanup
//!
//! ## Usage Examples
//!
//! ### Basic Setup
//! ```rust,no_run
//! use autodebugger::rotating_file_logger::RotatingFileLogger;
//! 
//! let _guard = RotatingFileLogger::init()
//!     .with_directory("logs")
//!     .with_filename("myapp.log")
//!     .with_max_files(10)
//!     .with_max_size_mb(10)
//!     .build();
//! ```
//!
//! ### With Tracing Integration
//! ```rust,no_run
//! use autodebugger::{init_logging_with_rotating_file, RotatingFileConfig};
//!
//! let config = RotatingFileConfig {
//!     log_directory: "logs".to_string(),
//!     filename: "app.log".to_string(),
//!     max_files: 5,
//!     max_size_mb: 10,
//!     console_output: true,
//!     truncate_on_limit: true,
//! };
//!
//! let (_layer, _guard) = init_logging_with_rotating_file(Some("info"), Some(config), None);
//! ```
//!
//! ## File Naming Convention
//!
//! Each run creates a unique timestamped log file:
//! - Current log: `app_YYYYMMDD_HHMMSS.log` (new file per run)
//! - Latest symlink: `app_latest.log` (always points to current timestamped log)
//!
//! When size limit is exceeded:
//! - `truncate_on_limit: true` (default): stops logging, preserves history across runs
//! - `truncate_on_limit: false`: creates numbered backups within the same run
//!
//! ## Performance Considerations
//!
//! - File size checks are performed on each write
//! - Rotation is atomic but may cause brief write delays
//! - Consider rotation size based on write frequency
//! - Use async logging for high-throughput applications

use crate::config::RotatingFileConfig;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing_subscriber::fmt::MakeWriter;

/// Builder for configuring the rotating file logger
pub struct RotatingFileLoggerBuilder {
    config: RotatingFileConfig,
}

impl RotatingFileLoggerBuilder {
    pub fn with_directory<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.config.log_directory = dir.as_ref().to_string_lossy().to_string();
        self
    }

    pub fn with_filename<S: Into<String>>(mut self, filename: S) -> Self {
        self.config.filename = filename.into();
        self
    }

    pub fn with_max_files(mut self, max: usize) -> Self {
        self.config.max_files = max;
        self
    }

    pub fn with_max_size_mb(mut self, size: u64) -> Self {
        self.config.max_size_mb = size;
        self
    }

    pub fn with_console(mut self, enabled: bool) -> Self {
        self.config.console_output = enabled;
        self
    }

    pub fn with_truncate_on_limit(mut self, truncate: bool) -> Self {
        self.config.truncate_on_limit = truncate;
        self
    }

    /// Build and initialize the rotating file logger
    /// Returns a guard that should be kept alive for the duration of logging
    pub fn build(self) -> Result<RotatingFileGuard, std::io::Error> {
        RotatingFileLogger::initialize(self.config)
    }
}

/// Main rotating file logger struct
pub struct RotatingFileLogger;

impl RotatingFileLogger {
    /// Start building a rotating file logger configuration
    pub fn init() -> RotatingFileLoggerBuilder {
        RotatingFileLoggerBuilder {
            config: RotatingFileConfig::default(),
        }
    }

    /// Initialize with default configuration
    pub fn init_default() -> Result<RotatingFileGuard, std::io::Error> {
        Self::initialize(RotatingFileConfig::default())
    }

    /// Initialize the rotating file logger with given configuration
    fn initialize(config: RotatingFileConfig) -> Result<RotatingFileGuard, std::io::Error> {
        // Create log directory if it doesn't exist
        fs::create_dir_all(&config.log_directory)?;

        // Create the rotating writer
        let writer = RotatingWriter::new(config.clone())?;
        let writer = Arc::new(Mutex::new(writer));

        Ok(RotatingFileGuard { _writer: writer })
    }
}

/// Guard that keeps the rotating logger alive
/// Drop this to stop logging to files
pub struct RotatingFileGuard {
    _writer: Arc<Mutex<RotatingWriter>>,
}

/// The actual rotating file writer
struct RotatingWriter {
    config: RotatingFileConfig,
    current_file: fs::File,
    current_size: u64,
    log_path: PathBuf,
}

impl RotatingWriter {
    fn new(config: RotatingFileConfig) -> Result<Self, std::io::Error> {
        // Add timestamp to filename to create unique file per run
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let base_name = config.filename.trim_end_matches(".log");
        let timestamped_filename = format!("{}_{}.log", base_name, timestamp);
        let log_path = PathBuf::from(&config.log_directory).join(&timestamped_filename);
        
        // Create new file for this run (not append)
        let current_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_path)?;
        
        // Get current file size
        let current_size = current_file.metadata()?.len();

        let writer = Self {
            config,
            current_file,
            current_size,
            log_path,
        };
        
        // Create initial symlink to current log file
        let _ = writer.update_latest_symlink(); // Ignore errors, just log warnings
        
        Ok(writer)
    }

    /// Rotate log files: app.log -> app.log.1, app.log.1 -> app.log.2, etc.
    fn rotate(&mut self) -> Result<(), std::io::Error> {
        // Flush current file
        self.current_file.flush()?;

        // Rotate existing numbered files
        for i in (1..self.config.max_files).rev() {
            let old_path = self.log_path.with_extension(format!("log.{}", i));
            let new_path = self.log_path.with_extension(format!("log.{}", i + 1));
            
            if old_path.exists() {
                if i + 1 >= self.config.max_files {
                    // Delete the oldest file if we're at max
                    fs::remove_file(&old_path)?;
                } else {
                    fs::rename(&old_path, &new_path)?;
                }
            }
        }

        // Move current log to .1
        let backup_path = self.log_path.with_extension("log.1");
        fs::rename(&self.log_path, &backup_path)?;

        // Create new empty log file
        self.current_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.log_path)?;
        
        self.current_size = 0;
        
        // Update symlink to point to the new log file
        let _ = self.update_latest_symlink(); // Ignore errors, just log warnings
        
        Ok(())
    }

    fn should_rotate(&self) -> bool {
        self.current_size >= self.config.max_size_mb * 1024 * 1024
    }

    /// Update the "latest" symlink/copy to point to the current log file
    fn update_latest_symlink(&self) -> std::io::Result<()> {
        // Generate the latest symlink name based on the base filename
        let base_name = self.config.filename.trim_end_matches(".log");
        let latest_filename = format!("{}_latest.log", base_name);
        let latest_path = PathBuf::from(&self.config.log_directory).join(&latest_filename);
        
        // Remove existing symlink/file if it exists
        if latest_path.exists() {
            if let Err(e) = fs::remove_file(&latest_path) {
                tracing::warn!("Failed to remove existing latest symlink: {}", e);
                return Ok(()); // Continue without failing
            }
        }
        
        // Create new symlink/copy pointing to current log file
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            // Use just the filename for the symlink target since they're in the same directory
            let target_filename = self.log_path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(&self.config.filename);
            if let Err(e) = symlink(target_filename, &latest_path) {
                tracing::warn!("Failed to create symlink {}: {}", latest_filename, e);
            }
        }
        
        #[cfg(windows)]
        {
            if let Err(e) = fs::copy(&self.log_path, &latest_path) {
                tracing::warn!("Failed to create latest copy {}: {}", latest_filename, e);
            }
        }
        
        Ok(())
    }
}

impl Write for RotatingWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Check if we're at size limit
        if self.should_rotate() {
            if self.config.truncate_on_limit {
                // Truncate mode: stop logging when limit is reached
                tracing::warn!("Log size limit reached ({}MB), stopping logging for this run", self.config.max_size_mb);
                return Ok(buf.len()); // Pretend we wrote it to avoid errors
            } else {
                // Backup mode: rotate to numbered files
                self.rotate()?;
            }
        }

        let written = self.current_file.write(buf)?;
        self.current_size += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.current_file.flush()
    }
}

/// Wrapper to implement MakeWriter that integrates with tracing subscriber
pub struct RotatingWriterWrapper(Arc<Mutex<RotatingWriter>>);

impl RotatingWriterWrapper {
    /// Create a new rotating writer wrapper
    pub fn new(config: RotatingFileConfig) -> Result<Self, std::io::Error> {
        // Create log directory if it doesn't exist
        fs::create_dir_all(&config.log_directory)?;
        
        let writer = RotatingWriter::new(config)?;
        let writer = Arc::new(Mutex::new(writer));
        Ok(Self(writer))
    }
    
    /// Convert to a guard that keeps the writer alive
    pub fn into_guard(self) -> RotatingFileGuard {
        RotatingFileGuard { _writer: self.0 }
    }
}

impl Clone for RotatingWriterWrapper {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'a> MakeWriter<'a> for RotatingWriterWrapper {
    type Writer = RotatingWriterGuard<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        RotatingWriterGuard {
            writer: self.0.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Guard for thread-safe writing  
pub struct RotatingWriterGuard<'a> {
    writer: Arc<Mutex<RotatingWriter>>,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Write for RotatingWriterGuard<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.lock().unwrap().flush()
    }
}