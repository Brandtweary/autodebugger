//! Documentation validation for Rust modules - Ensuring code quality through documentation standards
//!
//! This module provides comprehensive functionality to validate that Rust source files
//! have appropriate documentation based on their complexity. It enforces documentation
//! standards by analyzing module size and documentation coverage, helping maintain
//! high-quality, well-documented codebases.
//!
//! ## Philosophy
//!
//! Good documentation is essential for maintainable code. This validator operates on the
//! principle that more complex modules require more extensive documentation. Simple utility
//! modules may need minimal documentation, while complex business logic requires thorough
//! explanation of design decisions, algorithms, and usage patterns.
//!
//! ## Validation Rules
//!
//! The validator applies different standards based on module complexity:
//!
//! ### Complex Modules (>200 lines by default)
//! - **Minimum Documentation**: 50 lines of module-level docs
//! - **Rationale**: Large modules contain significant logic requiring explanation
//! - **Expected Content**: Architecture overview, design decisions, usage examples
//!
//! ### Simple Modules (<200 lines)
//! - **No Minimum**: Documentation encouraged but not required
//! - **Rationale**: Simple modules are often self-explanatory
//! - **Best Practice**: Still add brief module docs explaining purpose
//!
//! ### Maximum Documentation (all modules)
//! - **Maximum**: 200 lines of documentation
//! - **Rationale**: Excessive documentation can be as harmful as insufficient
//! - **Guidance**: Keep docs concise and focused on essential information
//!
//! ## Detection Algorithm
//!
//! The validator specifically detects Rust's standard `//!` documentation format:
//! 1. Scans from the beginning of each file
//! 2. Counts consecutive `//!` lines
//! 3. Allows blank lines within documentation blocks
//! 4. Stops at the first non-documentation line
//!
//! Note: This validator does NOT process:
//! - Regular comments (`//` or `/* */`)
//! - Item-level documentation (`///`)
//! - Documentation in other formats
//!
//! ## Configuration
//!
//! All thresholds are configurable via `config.yaml`:
//! ```yaml
//! validate_docs:
//!   default_paths: ["src"]
//!   min_doc_lines_complex: 50
//!   max_doc_lines: 200
//!   complexity_threshold: 200
//!   ignore_patterns: ["**/tests/**", "**/examples/**"]
//! ```
//!
//! ## Usage Examples
//!
//! ### Command Line
//! ```bash
//! # Validate with default settings
//! autodebugger validate-docs
//!
//! # Validate specific directories with verbose output
//! autodebugger validate-docs src/ lib/ --verbose
//!
//! # Strict mode - treat warnings as errors
//! autodebugger validate-docs --strict
//! ```
//!
//! ### Programmatic Usage
//! ```rust
//! use autodebugger::validate_docs::DocValidator;
//!
//! let validator = DocValidator::new()
//!     .with_min_doc_lines(60)
//!     .with_complexity_threshold(150)
//!     .with_ignore_patterns(vec!["**/generated/**".to_string()])?;
//!
//! let report = validator.validate_paths(vec!["src".into()])?;
//! if !report.passed(strict_mode) {
//!     eprintln!("Documentation validation failed!");
//! }
//! ```
//!
//! ## Integration with CI/CD
//!
//! The validator is designed for CI/CD integration:
//! - Exit code 0 on success (or warnings in non-strict mode)
//! - Exit code 1 on failure (warnings in strict mode)
//! - Machine-readable output for parsing
//! - Configurable via environment-specific config files
//!
//! ## Best Practices
//!
//! 1. **Start Early**: Add documentation as you write code
//! 2. **Be Concise**: Focus on "why" not "what"
//! 3. **Include Examples**: Show typical usage patterns
//! 4. **Document Decisions**: Explain non-obvious design choices
//! 5. **Update Regularly**: Keep docs in sync with code changes

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::warn;
use walkdir::WalkDir;
use glob::Pattern;

/// Documentation validator for Rust source files
pub struct DocValidator {
    /// Minimum documentation lines for complex modules
    pub min_doc_lines_complex: usize,
    /// Maximum documentation lines for any module
    pub max_doc_lines: usize,
    /// Line count threshold to consider a module "complex"
    pub complexity_threshold: usize,
    /// Glob patterns to ignore
    pub ignore_patterns: Vec<Pattern>,
    /// Whether to show verbose output
    pub verbose: bool,
    /// Whether to treat warnings as errors
    pub strict: bool,
}

impl DocValidator {
    /// Create a new validator with default configuration
    pub fn new() -> Self {
        Self {
            min_doc_lines_complex: 50,
            max_doc_lines: 200,
            complexity_threshold: 200,
            ignore_patterns: vec![],
            verbose: false,
            strict: false,
        }
    }

    /// Set minimum documentation lines for complex modules
    pub fn with_min_doc_lines(mut self, lines: usize) -> Self {
        self.min_doc_lines_complex = lines;
        self
    }

    /// Set maximum documentation lines
    pub fn with_max_doc_lines(mut self, lines: usize) -> Self {
        self.max_doc_lines = lines;
        self
    }

    /// Set complexity threshold
    pub fn with_complexity_threshold(mut self, lines: usize) -> Self {
        self.complexity_threshold = lines;
        self
    }

    /// Set ignore patterns
    pub fn with_ignore_patterns(mut self, patterns: Vec<String>) -> Result<Self> {
        let mut compiled_patterns = Vec::new();
        for pattern in patterns {
            let compiled = Pattern::new(&pattern)
                .with_context(|| format!("Invalid glob pattern: {}", pattern))?;
            compiled_patterns.push(compiled);
        }
        self.ignore_patterns = compiled_patterns;
        Ok(self)
    }

    /// Set verbose mode
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set strict mode
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Validate documentation for all Rust files in the given paths
    pub fn validate_paths(&self, paths: Vec<PathBuf>) -> Result<ValidationReport> {
        let mut report = ValidationReport::default();

        for path in paths {
            if path.is_file() {
                if self.should_process_file(&path) {
                    self.validate_file(&path, &mut report)?;
                }
            } else if path.is_dir() {
                self.validate_directory(&path, &mut report)?;
            } else {
                anyhow::bail!("Path does not exist: {}", path.display());
            }
        }

        Ok(report)
    }

    /// Check if a file should be processed based on ignore patterns
    fn should_process_file(&self, path: &Path) -> bool {
        // Only process .rs files
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            return false;
        }

        // Check ignore patterns
        let path_str = path.to_string_lossy();
        for pattern in &self.ignore_patterns {
            if pattern.matches(&path_str) {
                return false;
            }
        }

        true
    }

    /// Validate all Rust files in a directory
    fn validate_directory(&self, dir: &Path, report: &mut ValidationReport) -> Result<()> {
        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && self.should_process_file(path) {
                self.validate_file(path, report)?;
            }
        }
        Ok(())
    }

    /// Validate a single Rust file
    fn validate_file(&self, path: &Path, report: &mut ValidationReport) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let total_lines = content.lines().count();
        let doc_lines = self.count_module_doc_lines(&content);

        report.files_scanned += 1;

        // Determine if this is a complex module
        let is_complex = total_lines > self.complexity_threshold;

        // Track the file info for reporting
        let file_info = FileInfo {
            path: path.to_path_buf(),
            doc_lines,
            total_lines,
            is_complex,
        };

        // Check for validation issues
        let mut issues = Vec::new();

        // Only check complex modules
        if is_complex {
            // Warn if complex module has no header documentation at all
            if doc_lines == 0 {
                issues.push(ValidationIssue::NoDocs {
                    total_lines,
                });
            } else if doc_lines < self.min_doc_lines_complex {
                issues.push(ValidationIssue::InsufficientDocs {
                    lines: doc_lines,
                    min: self.min_doc_lines_complex,
                    total_lines,
                });
            }
        }
        
        // Check for excessive docs (applies to all modules)
        if doc_lines > self.max_doc_lines {
            issues.push(ValidationIssue::ExcessiveDocs {
                lines: doc_lines,
                max: self.max_doc_lines,
            });
        }

        if !issues.is_empty() {
            report.warnings += issues.len();
            
            if self.verbose {
                for issue in &issues {
                    match issue {
                        ValidationIssue::NoDocs { total_lines } => {
                            warn!(
                                "{}: Complex module ({} lines) has no documentation (use //! format)",
                                path.display(), total_lines
                            );
                        }
                        ValidationIssue::InsufficientDocs { lines, min, total_lines } => {
                            warn!(
                                "{}: Complex module ({} lines) has insufficient documentation ({} lines, minimum {})",
                                path.display(), total_lines, lines, min
                            );
                        }
                        ValidationIssue::ExcessiveDocs { lines, max } => {
                            warn!(
                                "{}: Excessive documentation ({} lines, maximum {})",
                                path.display(), lines, max
                            );
                        }
                    }
                }
            }

            report.file_issues.push((file_info, issues));
        } else if is_complex {
            // Only track complex modules that passed
            report.complex_files_passed.push(file_info);
        } else {
            // Track simple modules separately
            report.simple_files_skipped.push(file_info);
        }

        Ok(())
    }

    /// Count the number of module-level documentation lines (//!) at the start of a file
    fn count_module_doc_lines(&self, content: &str) -> usize {
        let mut doc_lines = 0;
        let mut in_doc_block = true;

        for line in content.lines() {
            let trimmed = line.trim();
            
            if in_doc_block {
                if trimmed.starts_with("//!") {
                    doc_lines += 1;
                } else if trimmed.is_empty() {
                    // Allow blank lines within documentation
                    continue;
                } else if trimmed.starts_with("#![") {
                    // Allow module-level attributes (like #![allow(dead_code)])
                    // These are valid at the module level and don't break the doc block
                    continue;
                } else if trimmed.starts_with("//") {
                    // Regular comments are allowed but don't count as docs
                    continue;
                } else {
                    // First actual code line (use statements, structs, etc.) - stop counting
                    in_doc_block = false;
                }
            } else {
                // Once we've left the doc block, we're done
                break;
            }
        }

        doc_lines
    }
}

/// Information about a file that was validated
#[derive(Debug)]
pub struct FileInfo {
    pub path: PathBuf,
    pub doc_lines: usize,
    pub total_lines: usize,
    pub is_complex: bool,
}

/// Report from documentation validation
#[derive(Debug, Default)]
pub struct ValidationReport {
    pub files_scanned: usize,
    pub warnings: usize,
    pub complex_files_passed: Vec<FileInfo>,
    pub simple_files_skipped: Vec<FileInfo>,
    pub file_issues: Vec<(FileInfo, Vec<ValidationIssue>)>,
}

impl ValidationReport {
    /// Print a summary of the validation results
    pub fn print_summary(&self, verbose: bool) {
        let complex_count = self.complex_files_passed.len() + self.file_issues.len();
        let simple_count = self.simple_files_skipped.len();
        
        // Non-verbose: Just show the essential summary
        if !verbose && self.warnings == 0 {
            println!("Validated {} files: {} complex, {} simple (skipped)", 
                    self.files_scanned, complex_count, simple_count);
            println!("✓ All complex modules have appropriate documentation!");
            return;
        }
        
        // Verbose mode or there are warnings - show detailed output
        println!("\nDocumentation Validation Report");
        println!("===============================");
        println!("{} files scanned: {} complex (validated), {} simple (skipped)",
                self.files_scanned, complex_count, simple_count);
        
        // Group results by status
        if !self.complex_files_passed.is_empty() && (verbose || self.warnings > 0) {
            println!("\n✅ Passed ({} complex modules):", self.complex_files_passed.len());
            if verbose {
                for file in &self.complex_files_passed {
                    println!("  {} ({} lines, {} doc lines)",
                            file.path.display(), file.total_lines, file.doc_lines);
                }
            } else {
                for file in &self.complex_files_passed {
                    println!("  {}", file.path.display());
                }
            }
        }
        
        if !self.file_issues.is_empty() {
            println!("\n⚠️  Warnings ({} modules):", self.file_issues.len());
            for (file_info, issues) in &self.file_issues {
                for issue in issues {
                    match issue {
                        ValidationIssue::NoDocs { total_lines } => {
                            println!("  {}: Complex module ({} lines) has no documentation (use //! format)",
                                    file_info.path.display(), total_lines);
                        }
                        ValidationIssue::InsufficientDocs { lines, min, total_lines } => {
                            println!("  {}: Complex module ({} lines) has insufficient documentation ({} lines, minimum {})",
                                    file_info.path.display(), total_lines, lines, min);
                        }
                        ValidationIssue::ExcessiveDocs { lines, max } => {
                            println!("  {}: Excessive documentation ({} lines, maximum {})",
                                    file_info.path.display(), lines, max);
                        }
                    }
                }
            }
        }
        
        if verbose && !self.simple_files_skipped.is_empty() {
            println!("\n⏭️  Skipped ({} simple modules under {} lines):", 
                    self.simple_files_skipped.len(), 
                    200); // TODO: get this from config
            for file in &self.simple_files_skipped {
                println!("  {} ({} lines)", file.path.display(), file.total_lines);
            }
        }
        
        // Final status for verbose or warning cases
        if verbose || self.warnings > 0 {
            if self.warnings == 0 && complex_count > 0 {
                println!("\n✓ All complex modules have appropriate documentation!");
            } else if self.warnings == 0 && complex_count == 0 {
                println!("\n✓ No complex modules found requiring validation.");
            } else {
                println!("\n❌ {} warning(s) found.", self.warnings);
            }
        }
    }

    /// Check if validation passed (no warnings in strict mode, otherwise always true)
    pub fn passed(&self, strict: bool) -> bool {
        !strict || self.warnings == 0
    }
}

/// Types of validation issues
#[derive(Debug)]
pub enum ValidationIssue {
    NoDocs {
        total_lines: usize,
    },
    InsufficientDocs { 
        lines: usize, 
        min: usize,
        total_lines: usize,
    },
    ExcessiveDocs { 
        lines: usize, 
        max: usize,
    },
}