use anyhow::{Context, Result};
use cmd_lib::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, error, info, trace};

pub mod monitor;
pub mod log_checker;
pub mod config;

// Re-export the main types for easy access
pub use log_checker::VerbosityCheckLayer;
pub use config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct Autodebugger {
    working_dir: PathBuf,
}

impl Autodebugger {
    pub fn new() -> Self {
        Self {
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }

    pub fn with_working_dir(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }

    pub fn set_working_dir(&mut self, dir: PathBuf) -> Result<()> {
        if !dir.exists() {
            anyhow::bail!("Directory does not exist: {}", dir.display());
        }
        self.working_dir = dir;
        Ok(())
    }

    pub fn run_command(&self, command: &str) -> Result<CommandResult> {
        info!("Running command: {}", command);
        debug!("Working directory: {}", self.working_dir.display());

        std::env::set_current_dir(&self.working_dir)
            .context("Failed to set working directory")?;

        let result = run_fun!(bash -c $command);

        match result {
            Ok(output) => {
                trace!("Command succeeded with output: {}", output);
                Ok(CommandResult {
                    stdout: output,
                    stderr: String::new(),
                    exit_code: 0,
                    success: true,
                })
            }
            Err(e) => {
                error!("Command failed: {}", e);
                Ok(CommandResult {
                    stdout: String::new(),
                    stderr: e.to_string(),
                    exit_code: 1,
                    success: false,
                })
            }
        }
    }

    pub fn run_command_with_input(&self, command: &str, input: &str) -> Result<CommandResult> {
        info!("Running command with input: {}", command);
        debug!("Input: {}", input);
        debug!("Working directory: {}", self.working_dir.display());

        std::env::set_current_dir(&self.working_dir)
            .context("Failed to set working directory")?;

        let result = run_fun!(echo $input | bash -c $command);

        match result {
            Ok(output) => {
                trace!("Command succeeded with output: {}", output);
                Ok(CommandResult {
                    stdout: output,
                    stderr: String::new(),
                    exit_code: 0,
                    success: true,
                })
            }
            Err(e) => {
                error!("Command failed: {}", e);
                Ok(CommandResult {
                    stdout: String::new(),
                    stderr: e.to_string(),
                    exit_code: 1,
                    success: false,
                })
            }
        }
    }

    pub fn run_commands_sequential(&self, commands: Vec<&str>) -> Result<Vec<CommandResult>> {
        let mut results = Vec::new();
        
        for command in commands {
            let result = self.run_command(command)?;
            if !result.success {
                info!("Command chain stopped at: {}", command);
                results.push(result);
                break;
            }
            results.push(result);
        }
        
        Ok(results)
    }

    pub async fn run_command_async(&self, command: &str) -> Result<CommandResult> {
        let command = command.to_string();
        let working_dir = self.working_dir.clone();
        
        tokio::task::spawn_blocking(move || {
            let debugger = Autodebugger::with_working_dir(working_dir);
            debugger.run_command(&command)
        })
        .await
        .context("Failed to spawn blocking task")?
    }
}

impl Default for Autodebugger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_command() {
        let debugger = Autodebugger::new();
        let result = debugger.run_command("echo 'Hello, World!'").unwrap();
        assert!(result.success);
        assert_eq!(result.stdout.trim(), "Hello, World!");
    }

    #[test]
    fn test_command_with_input() {
        let debugger = Autodebugger::new();
        let result = debugger
            .run_command_with_input("cat", "Hello from stdin")
            .unwrap();
        assert!(result.success);
        assert_eq!(result.stdout.trim(), "Hello from stdin");
    }

    #[test]
    fn test_failed_command() {
        let debugger = Autodebugger::new();
        let result = debugger.run_command("false").unwrap();
        assert!(!result.success);
        assert_eq!(result.exit_code, 1);
    }

    #[test]
    fn test_sequential_commands() {
        let debugger = Autodebugger::new();
        let commands = vec!["echo 'First'", "echo 'Second'", "echo 'Third'"];
        let results = debugger.run_commands_sequential(commands).unwrap();
        
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.success));
        assert_eq!(results[0].stdout.trim(), "First");
        assert_eq!(results[1].stdout.trim(), "Second");
        assert_eq!(results[2].stdout.trim(), "Third");
    }

    #[test]
    fn test_working_directory() {
        let mut debugger = Autodebugger::new();
        let temp_dir = std::env::temp_dir();
        
        // Change working directory
        debugger.set_working_dir(temp_dir.clone()).unwrap();
        
        // Run pwd command to verify
        let result = debugger.run_command("pwd").unwrap();
        assert!(result.success);
        assert_eq!(result.stdout.trim(), temp_dir.to_str().unwrap());
    }
}