//! Submit command - submit a bookmark stack as PRs

use jj_ryu::error::{Error, Result};
use jj_ryu::graph::build_change_graph;
use jj_ryu::platform::{create_platform_service, parse_repo_info};
use jj_ryu::repo::JjWorkspace;
use jj_ryu::submit::{
    analyze_submission, create_submission_plan, execute_submission, Phase, ProgressCallback,
    PushStatus,
};
use jj_ryu::types::PullRequest;
use async_trait::async_trait;
use std::path::Path;

/// CLI progress callback that prints to stdout
struct CliProgress;

#[async_trait]
impl ProgressCallback for CliProgress {
    async fn on_phase(&self, phase: Phase) {
        match phase {
            Phase::Analyzing => println!("Analyzing..."),
            Phase::Planning => println!("Planning..."),
            Phase::Pushing => println!("Pushing bookmarks..."),
            Phase::CreatingPrs => println!("Creating PRs..."),
            Phase::UpdatingPrs => println!("Updating PRs..."),
            Phase::AddingComments => println!("Updating stack comments..."),
            Phase::Complete => println!("Done!"),
        }
    }

    async fn on_bookmark_push(&self, bookmark: &str, status: PushStatus) {
        match status {
            PushStatus::Started => println!("  Pushing {bookmark}..."),
            PushStatus::Success => println!("  ✓ Pushed {bookmark}"),
            PushStatus::AlreadySynced => println!("  - {bookmark} already synced"),
            PushStatus::Failed(msg) => println!("  ✗ Failed to push {bookmark}: {msg}"),
        }
    }

    async fn on_pr_created(&self, bookmark: &str, pr: &PullRequest) {
        println!("  ✓ Created PR #{} for {}", pr.number, bookmark);
        println!("    {}", pr.html_url);
    }

    async fn on_pr_updated(&self, bookmark: &str, pr: &PullRequest) {
        println!("  ✓ Updated PR #{} for {}", pr.number, bookmark);
    }

    async fn on_error(&self, error: &Error) {
        eprintln!("Error: {error}");
    }

    async fn on_message(&self, message: &str) {
        println!("{message}");
    }
}

/// Run the submit command
pub async fn run_submit(
    path: &Path,
    bookmark: &str,
    remote: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    // Open workspace
    let mut workspace = JjWorkspace::open(path)?;

    // Get remotes and select one
    let remotes = workspace.git_remotes()?;
    if remotes.is_empty() {
        return Err(Error::NoSupportedRemotes);
    }

    let remote_name = if let Some(name) = remote {
        // User specified a remote
        if !remotes.iter().any(|r| r.name == name) {
            return Err(Error::RemoteNotFound(name.to_string()));
        }
        name.to_string()
    } else if remotes.len() == 1 {
        // Only one remote, use it
        remotes[0].name.clone()
    } else {
        // Default to "origin" if exists, otherwise first
        remotes
            .iter()
            .find(|r| r.name == "origin")
            .map_or_else(|| remotes[0].name.clone(), |r| r.name.clone())
    };

    // Detect platform from remote URL
    let remote_info = remotes
        .iter()
        .find(|r| r.name == remote_name)
        .ok_or_else(|| Error::RemoteNotFound(remote_name.clone()))?;

    let platform_config = parse_repo_info(&remote_info.url)?;

    // Create platform service
    let platform = create_platform_service(&platform_config).await?;

    // Build change graph
    let graph = build_change_graph(&workspace)?;

    if graph.bookmarks.is_empty() {
        println!("No bookmarks found in repository");
        return Ok(());
    }

    // Check if target bookmark exists
    if !graph.bookmarks.contains_key(bookmark) {
        return Err(Error::BookmarkNotFound(bookmark.to_string()));
    }

    // Analyze submission
    let analysis = analyze_submission(&graph, bookmark)?;

    println!(
        "Submitting {} bookmark{} in stack:",
        analysis.segments.len(),
        if analysis.segments.len() == 1 { "" } else { "s" }
    );
    for segment in &analysis.segments {
        let synced = if segment.bookmark.is_synced {
            " (synced)"
        } else {
            ""
        };
        println!("  - {}{}", segment.bookmark.name, synced);
    }
    println!();

    // Get default branch
    let default_branch = workspace.default_branch()?;

    // Create submission plan
    let plan = create_submission_plan(&analysis, platform.as_ref(), &remote_name, &default_branch)
        .await?;

    // Execute plan
    let progress = CliProgress;
    let result = execute_submission(&plan, &mut workspace, platform.as_ref(), &progress, dry_run)
        .await?;

    // Summary
    if !dry_run {
        println!();
        if result.success {
            println!("Successfully submitted {} bookmark{}",
                analysis.segments.len(),
                if analysis.segments.len() == 1 { "" } else { "s" }
            );

            if !result.created_prs.is_empty() {
                println!("Created {} PR{}",
                    result.created_prs.len(),
                    if result.created_prs.len() == 1 { "" } else { "s" }
                );
            }
        } else {
            eprintln!("Submission failed");
            for err in &result.errors {
                eprintln!("  {err}");
            }
        }
    }

    Ok(())
}
