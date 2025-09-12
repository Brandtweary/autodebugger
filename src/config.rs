use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

/// Main configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub verbosity: VerbosityConfig,
    
    #[serde(default)]
    pub remove_debug: RemoveDebugConfig,
    
    #[serde(default)]
    pub validate_docs: ValidateDocsConfig,
}

/// Configuration for remove-debug command
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RemoveDebugConfig {
    /// Default paths to search when no path is specified
    #[serde(default = "default_remove_debug_paths")]
    pub default_paths: Vec<String>,
}

/// Configuration for validate-docs command
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidateDocsConfig {
    /// Default paths to validate when no path is specified
    #[serde(default = "default_validate_docs_paths")]
    pub default_paths: Vec<String>,
    
    /// Minimum documentation lines for complex modules
    #[serde(default = "default_min_doc_lines_complex")]
    pub min_doc_lines_complex: usize,
    
    /// Maximum documentation lines for any module
    #[serde(default = "default_max_doc_lines")]
    pub max_doc_lines: usize,
    
    /// Line count threshold to consider a module "complex"
    #[serde(default = "default_complexity_threshold")]
    pub complexity_threshold: usize,
    
    /// Glob patterns to ignore (e.g., "**/tests/**")
    #[serde(default = "default_ignore_patterns")]
    pub ignore_patterns: Vec<String>,
}


/// Log verbosity threshold configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VerbosityConfig {
    /// Threshold for INFO level logging
    #[serde(default = "default_info_threshold")]
    pub info_threshold: usize,
    
    /// Threshold for DEBUG level logging
    #[serde(default = "default_debug_threshold")]
    pub debug_threshold: usize,
    
    /// Threshold for TRACE level logging
    #[serde(default = "default_trace_threshold")]
    pub trace_threshold: usize,
}

/// File logging configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileLogConfig {
    /// Path to log file
    pub file_path: String,
    /// Whether to truncate file on startup (default: true)
    #[serde(default = "default_truncate")]
    pub truncate: bool,
}

/// Rotating file logger configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RotatingFileConfig {
    /// Directory to store log files (created if doesn't exist)
    #[serde(default = "default_log_directory")]
    pub log_directory: String,
    /// Base filename for logs (e.g., "app.log")
    #[serde(default = "default_log_filename")]
    pub filename: String,
    /// Maximum number of rotating files to keep (default: 10)
    #[serde(default = "default_max_files")]
    pub max_files: usize,
    /// Maximum file size in MB before rotation (default: 5)
    #[serde(default = "default_max_size_mb")]
    pub max_size_mb: u64,
    /// Whether to also output to console (default: true)
    #[serde(default = "default_console_output")]
    pub console_output: bool,
    /// Whether to truncate on size limit vs create numbered backups (default: true)
    #[serde(default = "default_truncate_on_limit")]
    pub truncate_on_limit: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            verbosity: VerbosityConfig::default(),
            remove_debug: RemoveDebugConfig::default(),
            validate_docs: ValidateDocsConfig::default(),
        }
    }
}

impl Default for RemoveDebugConfig {
    fn default() -> Self {
        Self {
            default_paths: default_remove_debug_paths(),
        }
    }
}

impl Default for ValidateDocsConfig {
    fn default() -> Self {
        Self {
            default_paths: default_validate_docs_paths(),
            min_doc_lines_complex: default_min_doc_lines_complex(),
            max_doc_lines: default_max_doc_lines(),
            complexity_threshold: default_complexity_threshold(),
            ignore_patterns: default_ignore_patterns(),
        }
    }
}


impl Default for VerbosityConfig {
    fn default() -> Self {
        Self {
            info_threshold: default_info_threshold(),
            debug_threshold: default_debug_threshold(),
            trace_threshold: default_trace_threshold(),
        }
    }
}

impl Default for FileLogConfig {
    fn default() -> Self {
        Self {
            file_path: "logs/app.log".to_string(),
            truncate: default_truncate(),
        }
    }
}

impl Default for RotatingFileConfig {
    fn default() -> Self {
        Self {
            log_directory: default_log_directory(),
            filename: default_log_filename(),
            max_files: default_max_files(),
            max_size_mb: default_max_size_mb(),
            console_output: default_console_output(),
            truncate_on_limit: default_truncate_on_limit(),
        }
    }
}

// Default threshold functions for serde
fn default_info_threshold() -> usize { 50 }
fn default_debug_threshold() -> usize { 100 }
fn default_trace_threshold() -> usize { 200 }
fn default_truncate() -> bool { true }
fn default_log_directory() -> String { "logs".to_string() }
fn default_log_filename() -> String { "app.log".to_string() }
fn default_max_files() -> usize { 10 }
fn default_max_size_mb() -> u64 { 5 }
fn default_console_output() -> bool { true }
fn default_truncate_on_limit() -> bool { true }
fn default_remove_debug_paths() -> Vec<String> { 
    vec!["src".to_string(), "tests".to_string()] 
}
fn default_validate_docs_paths() -> Vec<String> { 
    vec!["src".to_string()] 
}
fn default_min_doc_lines_complex() -> usize { 50 }
fn default_max_doc_lines() -> usize { 200 }
fn default_complexity_threshold() -> usize { 200 }
fn default_ignore_patterns() -> Vec<String> {
    vec!["**/tests/**".to_string(), "**/examples/**".to_string()]
}

impl Config {
    /// Load configuration from file, or use defaults if not found
    pub fn load() -> Result<Self> {
        // Try to load from config.yaml in current directory
        let config_path = Path::new("config.yaml");
        
        if config_path.exists() {
            let contents = fs::read_to_string(config_path)
                .context("Failed to read config.yaml")?;
            let config: Config = serde_yaml::from_str(&contents)
                .context("Failed to parse config.yaml")?;
            Ok(config)
        } else {
            // Use defaults if no config file exists
            Ok(Config::default())
        }
    }
    
    /// Load configuration from a specific file path
    pub fn load_from(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config from {:?}", path))?;
        let config: Config = serde_yaml::from_str(&contents)
            .with_context(|| format!("Failed to parse config from {:?}", path))?;
        Ok(config)
    }
}