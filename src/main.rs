use anyhow::Result;
use autodebugger::{
    Autodebugger, 
    monitor::Monitor, 
    remove_debug::DebugRemover,
    init_logging_with_file,
    RotatingFileConfig,
};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

#[derive(Parser)]
#[command(author, version, about = "Cybernetic coding dashboard for monitoring LLM agents", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Monitor worktrees for changes
    Monitor {
        /// Path to workspace containing worktrees
        path: PathBuf,
        
        /// Output format (json, text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    
    /// Show diffs across worktrees
    Diff {
        /// Specific worktree name (shows all if not specified)
        worktree: Option<String>,
        
        /// Show summary only
        #[arg(short, long)]
        summary: bool,
        
        /// Path to workspace
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    
    /// Get aggregated context
    Context {
        /// Context type (local-tasks, status)
        #[arg(default_value = "status")]
        context_type: String,
        
        /// Path to workspace
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    
    /// Show status of all worktrees
    Status {
        /// Path to workspace
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    
    /// Run a command (legacy mode)
    Run {
        /// Command to execute
        command: Vec<String>,
    },
    
    /// Remove all debug! macro calls from Rust source files
    RemoveDebug {
        /// Paths to files or directories (uses config defaults if none specified)
        paths: Vec<PathBuf>,
        
        /// Run in dry-run mode (preview changes without modifying files)
        #[arg(short = 'n', long)]
        dry_run: bool,
        
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}


#[tokio::main]
async fn main() -> Result<()> {
    // Initialize autodebugger's tracing subscriber with rotating file logging
    let file_config = RotatingFileConfig {
        log_directory: PathBuf::from("autodebugger_logs"),
        filename: "autodebugger.log".to_string(),
        max_files: 10,  // Keep 10 rotating logs
        max_size_mb: 5,  // Rotate at 5MB
        console_output: true,  // Also output to console
    };
    
    let (_verbosity_layer, _file_guard) = init_logging_with_file(
        Some("info"),
        Some(file_config),
    );
    
    info!("Autodebugger starting");
    
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Monitor { path, format }) => {
            info!("Starting monitor for path: {}", path.display());
            let monitor = Monitor::new(path)?;
            let status = monitor.status()?;
            
            match format.as_str() {
                "json" => println!("{}", serde_json::to_string_pretty(&status)?),
                _ => {
                    println!("Worktree Status Report");
                    println!("====================");
                    for (name, worktree) in &status.worktrees {
                        println!("\n{}: {} ({})", name, worktree.status, worktree.branch);
                        if let Some(task) = &worktree.current_task {
                            println!("  Current task: {}", task);
                        }
                        println!("  Files changed: {}", worktree.files_changed);
                        if let Some(last) = &worktree.last_change {
                            println!("  Last change: {}", last);
                        }
                    }
                }
            }
        }
        
        Some(Commands::Diff { worktree, summary, path }) => {
            let monitor = Monitor::new(path)?;
            
            if summary {
                // Summary mode implementation pending
                println!("Summary mode not yet implemented");
            } else {
                let diff = monitor.diff(worktree.as_deref())?;
                println!("{}", diff);
            }
        }
        
        Some(Commands::Context { context_type, path }) => {
            let monitor = Monitor::new(path)?;
            let context = monitor.context(&context_type)?;
            println!("{}", context);
        }
        
        Some(Commands::Status { path, json }) => {
            let monitor = Monitor::new(path)?;
            let status = monitor.status()?;
            
            if json {
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else {
                for (name, worktree) in &status.worktrees {
                    println!("{}: {} ({} files changed)", 
                        name, 
                        worktree.status,
                        worktree.files_changed
                    );
                }
            }
        }
        
        Some(Commands::Run { command }) => {
            // Legacy command execution mode
            let command_str = command.join(" ");
            let debugger = Autodebugger::new();
            let result = debugger.run_command(&command_str)?;
            
            if !result.stdout.is_empty() {
                print!("{}", result.stdout);
            }
            if !result.stderr.is_empty() {
                eprint!("{}", result.stderr);
            }
            std::process::exit(result.exit_code);
        }
        
        Some(Commands::RemoveDebug { paths, dry_run, verbose }) => {
            use autodebugger::config::Config;
            
            // Use provided paths or fall back to config defaults
            let paths_to_process = if paths.is_empty() {
                let config = Config::load().unwrap_or_default();
                config.remove_debug.default_paths.into_iter()
                    .map(PathBuf::from)
                    .collect()
            } else {
                paths
            };
            
            if dry_run && verbose {
                info!("Running in dry-run mode - no files will be modified");
            }
            
            let mut total_report = autodebugger::remove_debug::RemovalReport::default();
            
            for path in paths_to_process {
                if verbose {
                    info!("Processing path: {}", path.display());
                }
                
                let remover = DebugRemover::new(path)
                    .with_dry_run(dry_run)
                    .with_verbose(verbose);
                
                let report = remover.remove_debug_calls()?;
                
                // Aggregate reports
                total_report.files_scanned += report.files_scanned;
                total_report.files_modified += report.files_modified;
                total_report.total_lines_removed += report.total_lines_removed;
                total_report.total_warnings += report.total_warnings;
                total_report.file_reports.extend(report.file_reports);
            }
            
            total_report.print_summary(verbose);
            
            if dry_run && total_report.total_lines_removed > 0 {
                info!("Re-run without --dry-run to apply changes");
            }
        }
        
        None => {
            // No command specified, show help
            println!("Autodebugger - Cybernetic Coding Dashboard");
            println!("\nUse --help for usage information");
        }
    }
    
    info!("Autodebugger shutting down");
    Ok(())
}