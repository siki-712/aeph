mod app;
mod markdown;
mod storage;
mod task;
mod ui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ae")]
#[command(about = "aeph - A TUI markdown paper with task management")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to markdown file (opens TUI editor if provided)
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Format a markdown file
    Fmt {
        /// File to format
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Write changes in-place
        #[arg(short, long)]
        write: bool,
    },
    /// Manage tasks in a markdown file
    Task {
        #[command(subcommand)]
        action: TaskCommands,
    },
}

#[derive(Subcommand)]
enum TaskCommands {
    /// List all tasks
    List {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Show only incomplete tasks
        #[arg(short, long)]
        pending: bool,
    },
    /// Add a new task
    Add {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Task description
        text: String,
    },
    /// Toggle task completion
    Toggle {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Task number (1-based)
        number: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Fmt { file, write }) => {
            markdown::format_file(&file, write)?;
        }
        Some(Commands::Task { action }) => match action {
            TaskCommands::List { file, pending } => {
                task::list_tasks(&file, pending)?;
            }
            TaskCommands::Add { file, text } => {
                task::add_task(&file, &text)?;
            }
            TaskCommands::Toggle { file, number } => {
                task::toggle_task(&file, number)?;
            }
        },
        None => {
            // Launch TUI
            app::run(cli.file)?;
        }
    }

    Ok(())
}
