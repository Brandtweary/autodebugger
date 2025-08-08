use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use regex::Regex;

pub struct DebugRemover {
    /// Path to search for Rust files
    pub path: PathBuf,
    /// Whether to run in dry-run mode (preview only)
    pub dry_run: bool,
    /// Whether to show verbose output
    pub verbose: bool,
}

impl DebugRemover {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            dry_run: false,
            verbose: false,
        }
    }

    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Remove all debug! macro calls from Rust files in the given path
    pub fn remove_debug_calls(&self) -> Result<RemovalReport> {
        let mut report = RemovalReport::default();
        
        if self.path.is_file() {
            self.process_file(&self.path, &mut report)?;
        } else if self.path.is_dir() {
            self.process_directory(&self.path, &mut report)?;
        } else {
            anyhow::bail!("Path does not exist: {}", self.path.display());
        }
        
        Ok(report)
    }

    fn process_directory(&self, dir: &Path, report: &mut RemovalReport) -> Result<()> {
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
                self.process_file(path, report)?;
            }
        }
        Ok(())
    }

    fn process_file(&self, path: &Path, report: &mut RemovalReport) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        
        let (new_content, file_report) = self.remove_debug_from_content(&content);
        
        if file_report.lines_removed > 0 {
            report.files_modified += 1;
            report.total_lines_removed += file_report.lines_removed;
            report.total_warnings += file_report.warnings.len();
            
            if self.verbose {
                info!("Processing {}", path.display());
                if file_report.lines_removed > 0 {
                    info!("  Removed {} debug! call(s)", file_report.lines_removed);
                }
                for warning in &file_report.warnings {
                    warn!("  Line {}: {}", warning.line_number, warning.message);
                }
            }
            
            if !self.dry_run {
                fs::write(path, new_content)
                    .with_context(|| format!("Failed to write file: {}", path.display()))?;
            }
            
            report.file_reports.push((path.to_path_buf(), file_report));
        }
        
        report.files_scanned += 1;
        Ok(())
    }

    fn remove_debug_from_content(&self, content: &str) -> (String, FileReport) {
        let mut new_lines = Vec::new();
        let mut report = FileReport::default();
        
        // Regex for simple, standalone debug! calls
        // Matches lines that contain only whitespace, debug!, and its arguments
        // Allows trailing comments after the semicolon
        let simple_debug_re = Regex::new(r"^\s*debug!\s*\([^;]*\)\s*;\s*(?://.*)?$").unwrap();
        
        // Regex to detect debug! anywhere in a line (for warning purposes)
        let any_debug_re = Regex::new(r"debug!\s*\(").unwrap();
        
        // Track if we're in a multiline comment
        let mut in_block_comment = false;
        
        for (line_num, line) in content.lines().enumerate() {
            let line_number = line_num + 1;
            
            // Check for block comment boundaries
            if line.contains("/*") {
                in_block_comment = true;
            }
            
            // Check if line contains debug!
            if any_debug_re.is_match(line) {
                // Case 1: Line is entirely a simple debug! call - remove it
                if simple_debug_re.is_match(line) && !in_block_comment && !line.trim_start().starts_with("//") {
                    report.lines_removed += 1;
                    continue; // Skip this line entirely
                }
                
                // Case 2: debug! in a comment
                if in_block_comment || line.trim_start().starts_with("//") {
                    report.warnings.push(Warning {
                        line_number,
                        message: "debug! found in comment - skipping".to_string(),
                    });
                    new_lines.push(line.to_string());
                }
                // Case 3: debug! with other code on the same line
                else if !simple_debug_re.is_match(line) {
                    report.warnings.push(Warning {
                        line_number,
                        message: "debug! found with other code on same line - skipping".to_string(),
                    });
                    new_lines.push(line.to_string());
                }
            } else {
                // No debug! on this line, keep it
                new_lines.push(line.to_string());
            }
            
            if line.contains("*/") {
                in_block_comment = false;
            }
        }
        
        (new_lines.join("\n"), report)
    }
}

#[derive(Debug, Default)]
pub struct RemovalReport {
    pub files_scanned: usize,
    pub files_modified: usize,
    pub total_lines_removed: usize,
    pub total_warnings: usize,
    pub file_reports: Vec<(PathBuf, FileReport)>,
}

impl RemovalReport {
    pub fn print_summary(&self, verbose: bool) {
        if verbose {
            info!("=== Debug Removal Summary ===");
            info!("Files scanned: {}", self.files_scanned);
            info!("Files modified: {}", self.files_modified);
            info!("Lines removed: {}", self.total_lines_removed);
            if self.total_warnings > 0 {
                info!("Warnings: {}", self.total_warnings);
            }
        } else {
            // Quiet mode: single line output
            if self.total_lines_removed > 0 {
                info!("Removed {} debug! calls from {} files", self.total_lines_removed, self.files_modified);
            } else {
                info!("No debug! calls found");
            }
            if self.total_warnings > 0 {
                info!("Skipped {} ambiguous cases (use --verbose for details)", self.total_warnings);
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct FileReport {
    pub lines_removed: usize,
    pub warnings: Vec<Warning>,
}

#[derive(Debug)]
pub struct Warning {
    pub line_number: usize,
    pub message: String,
}