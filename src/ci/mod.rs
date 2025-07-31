use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use serde::{Serialize, Deserialize};

pub mod checks;
pub mod conflicts;

use checks::CheckRunner;
use conflicts::ConflictAnalyzer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CIReport {
    pub worktree: String,
    pub checks: CheckResults,
    pub conflicts: Vec<ConflictPrediction>,
    pub safety_score: u8,
    pub recommendation: MergeRecommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResults {
    pub cargo_check: CheckStatus,
    pub cargo_test: CheckStatus,
    pub cargo_clippy: CheckStatus,
    pub debug_macros: CheckStatus,
    pub todo_comments: CheckStatus,
    pub documentation: CheckStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckStatus {
    Pass,
    Fail(String),
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictPrediction {
    pub file: String,
    pub overlapping_lines: usize,
    pub severity: ConflictSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictSeverity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeRecommendation {
    Safe,
    Caution(String),
    Danger(String),
}

pub struct CI {
    check_runner: CheckRunner,
    conflict_analyzer: ConflictAnalyzer,
}

impl CI {
    pub fn new() -> Self {
        Self {
            check_runner: CheckRunner::new(),
            conflict_analyzer: ConflictAnalyzer::new(),
        }
    }
    
    pub fn pre_merge_checks(&self, worktree_path: &Path) -> Result<CheckResults> {
        self.check_runner.run_all_checks(worktree_path)
    }
    
    pub fn analyze_conflicts(&self, base: &Path, incoming: &Path) -> Result<Vec<ConflictPrediction>> {
        self.conflict_analyzer.predict_conflicts(base, incoming)
    }
    
    pub fn calculate_safety_score(&self, checks: &CheckResults, conflicts: &[ConflictPrediction]) -> u8 {
        let mut score = 100u8;
        
        // Deduct for failed checks
        if matches!(checks.cargo_check, CheckStatus::Fail(_)) { score -= 30; }
        if matches!(checks.cargo_test, CheckStatus::Fail(_)) { score -= 20; }
        if matches!(checks.cargo_clippy, CheckStatus::Fail(_)) { score -= 10; }
        if matches!(checks.debug_macros, CheckStatus::Fail(_)) { score -= 5; }
        
        // Deduct for conflicts
        for conflict in conflicts {
            match conflict.severity {
                ConflictSeverity::High => score = score.saturating_sub(15),
                ConflictSeverity::Medium => score = score.saturating_sub(10),
                ConflictSeverity::Low => score = score.saturating_sub(5),
            }
        }
        
        score
    }
    
    pub fn generate_recommendation(&self, score: u8, conflicts: &[ConflictPrediction]) -> MergeRecommendation {
        if score >= 80 && conflicts.is_empty() {
            MergeRecommendation::Safe
        } else if score >= 60 {
            MergeRecommendation::Caution(
                format!("Review {} conflicts before merging", conflicts.len())
            )
        } else {
            MergeRecommendation::Danger(
                "Multiple issues detected. Manual review required.".to_string()
            )
        }
    }
}