//! ryu - Stacked PRs for Jujutsu
//!
//! CLI binary for managing stacked pull requests with jj.

use anyhow::Result;
use clap::{Parser, Subcommand};
use jj_ryu::types::Platform;
use std::path::PathBuf;

mod cli;

#[derive(Parser)]
#[command(name = "ryu")]
#[command(about = "Stacked PRs for Jujutsu - GitHub & GitLab")]
#[command(version)]
struct Cli {
    /// Path to jj repository (defaults to current directory)
    #[arg(short, long, global = true)]
    path: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Submit a bookmark stack as PRs
    Submit {
        /// Bookmark name to submit
        bookmark: String,

        /// Dry run - show what would be done without making changes
        #[arg(long)]
        dry_run: bool,

        /// Git remote to push to
        #[arg(long)]
        remote: Option<String>,
    },

    /// Sync all stacks with remote
    Sync {
        /// Dry run - show what would be done without making changes
        #[arg(long)]
        dry_run: bool,

        /// Git remote to sync with
        #[arg(long)]
        remote: Option<String>,
    },

    /// Authentication management
    Auth {
        #[command(subcommand)]
        platform: AuthPlatform,
    },
}

#[derive(Subcommand)]
enum AuthPlatform {
    /// GitHub authentication
    Github {
        #[command(subcommand)]
        action: AuthAction,
    },
    /// GitLab authentication
    Gitlab {
        #[command(subcommand)]
        action: AuthAction,
    },
}

#[derive(Subcommand)]
enum AuthAction {
    /// Test authentication
    Test,
    /// Show authentication setup instructions
    Setup,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let path = cli.path.unwrap_or_else(|| PathBuf::from("."));

    match cli.command {
        None => {
            // Default: interactive mode
            cli::run_analyze(&path).await?;
        }
        Some(Commands::Submit {
            bookmark,
            dry_run,
            remote,
        }) => {
            cli::run_submit(&path, &bookmark, remote.as_deref(), dry_run).await?;
        }
        Some(Commands::Sync { dry_run, remote }) => {
            cli::run_sync(&path, remote.as_deref(), dry_run).await?;
        }
        Some(Commands::Auth { platform }) => match platform {
            AuthPlatform::Github { action } => {
                let action_str = match action {
                    AuthAction::Test => "test",
                    AuthAction::Setup => "setup",
                };
                cli::run_auth(Platform::GitHub, action_str).await?;
            }
            AuthPlatform::Gitlab { action } => {
                let action_str = match action {
                    AuthAction::Test => "test",
                    AuthAction::Setup => "setup",
                };
                cli::run_auth(Platform::GitLab, action_str).await?;
            }
        },
    }

    Ok(())
}
