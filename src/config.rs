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
    
    /// Number of rotating log files to keep (default: 10)
    #[serde(default = "default_log_rotation_count")]
    pub log_rotation_count: usize,
}

/// Configuration for remove-debug command
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RemoveDebugConfig {
    /// Default paths to search when no path is specified
    #[serde(default = "default_remove_debug_paths")]
    pub default_paths: Vec<String>,
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

impl Default for Config {
    fn default() -> Self {
        Self {
            verbosity: VerbosityConfig::default(),
            remove_debug: RemoveDebugConfig::default(),
            log_rotation_count: default_log_rotation_count(),
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

impl Default for VerbosityConfig {
    fn default() -> Self {
        Self {
            info_threshold: default_info_threshold(),
            debug_threshold: default_debug_threshold(),
            trace_threshold: default_trace_threshold(),
        }
    }
}

// Default threshold functions for serde
fn default_info_threshold() -> usize { 50 }
fn default_debug_threshold() -> usize { 100 }
fn default_trace_threshold() -> usize { 200 }
fn default_remove_debug_paths() -> Vec<String> { 
    vec!["src".to_string(), "tests".to_string()] 
}
fn default_log_rotation_count() -> usize { 10 }

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