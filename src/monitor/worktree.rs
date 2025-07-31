use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::monitor::WorktreeStatus;

#[derive(Debug, Clone)]
pub struct Worktree {
    pub name: String,
    pub path: PathBuf,
}

pub struct WorktreeMonitor {
    workspace_path: PathBuf,
}

impl WorktreeMonitor {
    pub fn new(workspace_path: PathBuf) -> Result<Self> {
        if !workspace_path.exists() {
            anyhow::bail!("Workspace path does not exist: {}", workspace_path.display());
        }
        Ok(Self { workspace_path })
    }
    
    pub fn scan_worktrees(&self) -> Result<Vec<Worktree>> {
        let worktrees_dir = self.workspace_path.join("worktrees");
        if !worktrees_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut worktrees = Vec::new();
        
        for entry in std::fs::read_dir(&worktrees_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                    
                // Check if it's a git worktree
                if path.join(".git").exists() {
                    worktrees.push(Worktree { name, path });
                }
            }
        }
        
        Ok(worktrees)
    }
    
    pub fn get_status(&self, worktree: &Worktree) -> Result<WorktreeStatus> {
        // Get git branch
        let branch = self.get_git_branch(&worktree.path)?;
        
        // Get git status
        let git_status = Command::new("git")
            .current_dir(&worktree.path)
            .args(&["status", "--porcelain"])
            .output()
            .context("Failed to run git status")?;
            
        let files_changed = String::from_utf8_lossy(&git_status.stdout)
            .lines()
            .count();
            
        // Get last commit time
        let last_commit = Command::new("git")
            .current_dir(&worktree.path)
            .args(&["log", "-1", "--format=%ar"])
            .output()
            .context("Failed to get last commit")?;
            
        let last_change = if last_commit.status.success() {
            Some(String::from_utf8_lossy(&last_commit.stdout).trim().to_string())
        } else {
            None
        };
        
        // Try to extract current task from CLAUDE.local.md
        let current_task = self.extract_current_task(&worktree.path);
        
        Ok(WorktreeStatus {
            name: worktree.name.clone(),
            path: worktree.path.clone(),
            status: if files_changed > 0 { "active".to_string() } else { "idle".to_string() },
            last_change,
            files_changed,
            current_task,
            branch,
        })
    }
    
    fn get_git_branch(&self, path: &Path) -> Result<String> {
        let output = Command::new("git")
            .current_dir(path)
            .args(&["branch", "--show-current"])
            .output()
            .context("Failed to get git branch")?;
            
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
    
    fn extract_current_task(&self, path: &Path) -> Option<String> {
        let claude_local = path.join("CLAUDE.local.md");
        if !claude_local.exists() {
            return None;
        }
        
        // Simple extraction: look for first task in "## Specific Tasks" section
        if let Ok(content) = std::fs::read_to_string(&claude_local) {
            let lines: Vec<&str> = content.lines().collect();
            let mut in_tasks = false;
            
            for line in lines {
                if line.starts_with("## Specific Tasks") {
                    in_tasks = true;
                } else if in_tasks && line.starts_with("1.") {
                    return Some(line.trim_start_matches("1.").trim().to_string());
                } else if in_tasks && line.starts_with("##") {
                    break; // End of tasks section
                }
            }
        }
        
        None
    }
}