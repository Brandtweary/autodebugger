use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub struct DiffTracker;

impl DiffTracker {
    pub fn new() -> Self {
        Self
    }
    
    pub fn get_diff_for_worktree(&self, workspace_path: &Path, worktree_name: &str) -> Result<String> {
        let worktree_path = workspace_path.join("worktrees").join(worktree_name);
        
        if !worktree_path.exists() {
            anyhow::bail!("Worktree not found: {}", worktree_name);
        }
        
        // Get both staged and unstaged changes
        let staged = self.get_staged_diff(&worktree_path)?;
        let unstaged = self.get_unstaged_diff(&worktree_path)?;
        
        let mut result = format!("# Diff for worktree: {}\n\n", worktree_name);
        
        if !staged.is_empty() {
            result.push_str("## Staged Changes\n\n");
            result.push_str(&staged);
            result.push_str("\n\n");
        }
        
        if !unstaged.is_empty() {
            result.push_str("## Unstaged Changes\n\n");
            result.push_str(&unstaged);
            result.push_str("\n\n");
        }
        
        if staged.is_empty() && unstaged.is_empty() {
            result.push_str("No changes detected.\n");
        }
        
        Ok(result)
    }
    
    pub fn get_all_diffs(&self, workspace_path: &Path) -> Result<String> {
        let worktrees_dir = workspace_path.join("worktrees");
        if !worktrees_dir.exists() {
            return Ok("No worktrees found.".to_string());
        }
        
        let mut all_diffs = Vec::new();
        
        for entry in std::fs::read_dir(&worktrees_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() && path.join(".git").exists() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                    
                match self.get_diff_for_worktree(workspace_path, name) {
                    Ok(diff) => all_diffs.push(diff),
                    Err(e) => all_diffs.push(format!("# Error getting diff for {}: {}\n", name, e)),
                }
            }
        }
        
        Ok(all_diffs.join("\n---\n\n"))
    }
    
    // Summary mode implementation pending
    // Should return: Added files, Modified files, Deleted files, Line counts
    pub fn get_diff_summary(&self, workspace_path: &Path, worktree_name: &str) -> Result<String> {
        let worktree_path = workspace_path.join("worktrees").join(worktree_name);
        
        if !worktree_path.exists() {
            anyhow::bail!("Worktree not found: {}", worktree_name);
        }
        
        // Get file status
        let output = Command::new("git")
            .current_dir(&worktree_path)
            .args(&["status", "--porcelain"])
            .output()
            .context("Failed to run git status")?;
            
        let status_lines = String::from_utf8_lossy(&output.stdout);
        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut deleted = Vec::new();
        
        for line in status_lines.lines() {
            if line.len() < 3 {
                continue;
            }
            
            let status = &line[0..2];
            let file = &line[3..];
            
            match status {
                "A " | " A" | "??" => added.push(file),
                "M " | " M" | "MM" => modified.push(file),
                "D " | " D" => deleted.push(file),
                _ => {}
            }
        }
        
        let mut summary = Vec::new();
        
        if !added.is_empty() {
            summary.push(format!("Added: {}", added.join(", ")));
        }
        if !modified.is_empty() {
            summary.push(format!("Modified: {}", modified.join(", ")));
        }
        if !deleted.is_empty() {
            summary.push(format!("Deleted: {}", deleted.join(", ")));
        }
        
        if summary.is_empty() {
            Ok("No changes".to_string())
        } else {
            Ok(summary.join("\n"))
        }
    }
    
    fn get_staged_diff(&self, path: &Path) -> Result<String> {
        let output = Command::new("git")
            .current_dir(path)
            .args(&["diff", "--cached"])
            .output()
            .context("Failed to get staged diff")?;
            
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    fn get_unstaged_diff(&self, path: &Path) -> Result<String> {
        let output = Command::new("git")
            .current_dir(path)
            .args(&["diff"])
            .output()
            .context("Failed to get unstaged diff")?;
            
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}