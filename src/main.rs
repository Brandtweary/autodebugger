use anyhow::Result;
use autodebugger::{Autodebugger, monitor::Monitor};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Monitor { path, format }) => {
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
                // TODO: Implement summary mode
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
        
        None => {
            // No command specified, show help
            println!("Autodebugger - Cybernetic Coding Dashboard");
            println!("\nUse --help for usage information");
        }
    }
    
    Ok(())
}