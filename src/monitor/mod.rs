use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;

pub mod worktree;
pub mod diff;

use worktree::WorktreeMonitor;
use diff::DiffTracker;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorStatus {
    pub worktrees: HashMap<String, WorktreeStatus>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeStatus {
    pub name: String,
    pub path: PathBuf,
    pub status: String,
    pub last_change: Option<String>,
    pub files_changed: usize,
    pub current_task: Option<String>,
    pub branch: String,
}

pub struct Monitor {
    workspace_path: PathBuf,
    worktree_monitor: WorktreeMonitor,
    diff_tracker: DiffTracker,
}

impl Monitor {
    pub fn new(workspace_path: PathBuf) -> Result<Self> {
        let worktree_monitor = WorktreeMonitor::new(workspace_path.clone())?;
        let diff_tracker = DiffTracker::new();
        
        Ok(Self {
            workspace_path,
            worktree_monitor,
            diff_tracker,
        })
    }
    
    pub fn status(&self) -> Result<MonitorStatus> {
        let worktrees = self.worktree_monitor.scan_worktrees()?;
        let mut status_map = HashMap::new();
        
        for worktree in worktrees {
            let name = worktree.name.clone();
            let worktree_status = self.worktree_monitor.get_status(&worktree)?;
            status_map.insert(name, worktree_status);
        }
        
        Ok(MonitorStatus {
            worktrees: status_map,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
    
    pub fn diff(&self, worktree_name: Option<&str>) -> Result<String> {
        match worktree_name {
            Some(name) => self.diff_tracker.get_diff_for_worktree(&self.workspace_path, name),
            None => self.diff_tracker.get_all_diffs(&self.workspace_path),
        }
    }
    
    pub fn context(&self, context_type: &str) -> Result<String> {
        match context_type {
            "local-tasks" => self.aggregate_local_tasks(),
            "status" => Ok(serde_json::to_string_pretty(&self.status()?)?),
            _ => anyhow::bail!("Unknown context type: {}", context_type),
        }
    }
    
    fn aggregate_local_tasks(&self) -> Result<String> {
        let worktrees = self.worktree_monitor.scan_worktrees()?;
        let mut tasks = Vec::new();
        
        for worktree in worktrees {
            let claude_local_path = worktree.path.join("CLAUDE.local.md");
            if claude_local_path.exists() {
                let content = std::fs::read_to_string(&claude_local_path)
                    .context("Failed to read CLAUDE.local.md")?;
                tasks.push(format!("## Worktree: {}\n\n{}", worktree.name, content));
            }
        }
        
        Ok(tasks.join("\n\n---\n\n"))
    }
}