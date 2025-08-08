use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

/// Main configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub verbosity: VerbosityConfig,
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
    
    /// Threshold for WARN level logging
    #[serde(default = "default_warn_threshold")]
    pub warn_threshold: usize,
    
    /// Threshold for ERROR level logging
    #[serde(default = "default_error_threshold")]
    pub error_threshold: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            verbosity: VerbosityConfig::default(),
        }
    }
}

impl Default for VerbosityConfig {
    fn default() -> Self {
        Self {
            info_threshold: default_info_threshold(),
            debug_threshold: default_debug_threshold(),
            trace_threshold: default_trace_threshold(),
            warn_threshold: default_warn_threshold(),
            error_threshold: default_error_threshold(),
        }
    }
}

// Default threshold functions for serde
fn default_info_threshold() -> usize { 50 }
fn default_debug_threshold() -> usize { 100 }
fn default_trace_threshold() -> usize { 200 }
fn default_warn_threshold() -> usize { 25 }
fn default_error_threshold() -> usize { 10 }

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