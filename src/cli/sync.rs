//! Sync command - sync all stacks with remote

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
            Phase::Pushing => println!("  Pushing bookmarks..."),
            Phase::CreatingPrs => println!("  Creating PRs..."),
            Phase::UpdatingPrs => println!("  Updating PRs..."),
            Phase::AddingComments => println!("  Updating comments..."),
            _ => {}
        }
    }

    async fn on_bookmark_push(&self, bookmark: &str, status: PushStatus) {
        match status {
            PushStatus::Started => print!("    Pushing {bookmark}... "),
            PushStatus::Success => println!("done"),
            PushStatus::AlreadySynced => println!("already synced"),
            PushStatus::Failed(msg) => println!("failed: {msg}"),
        }
    }

    async fn on_pr_created(&self, bookmark: &str, pr: &PullRequest) {
        println!("    Created PR #{} for {} ({})", pr.number, bookmark, pr.html_url);
    }

    async fn on_pr_updated(&self, bookmark: &str, pr: &PullRequest) {
        println!("    Updated PR #{} for {}", pr.number, bookmark);
    }

    async fn on_error(&self, error: &Error) {
        eprintln!("    Error: {error}");
    }

    async fn on_message(&self, message: &str) {
        println!("  {message}");
    }
}

/// Run the sync command
pub async fn run_sync(path: &Path, remote: Option<&str>, dry_run: bool) -> Result<()> {
    // Open workspace
    let mut workspace = JjWorkspace::open(path)?;

    // Get remotes and select one
    let remotes = workspace.git_remotes()?;
    if remotes.is_empty() {
        return Err(Error::NoSupportedRemotes);
    }

    let remote_name = if let Some(name) = remote {
        if !remotes.iter().any(|r| r.name == name) {
            return Err(Error::RemoteNotFound(name.to_string()));
        }
        name.to_string()
    } else if remotes.len() == 1 {
        remotes[0].name.clone()
    } else {
        remotes
            .iter()
            .find(|r| r.name == "origin")
            .map_or_else(|| remotes[0].name.clone(), |r| r.name.clone())
    };

    // Detect platform
    let remote_info = remotes
        .iter()
        .find(|r| r.name == remote_name)
        .ok_or_else(|| Error::RemoteNotFound(remote_name.clone()))?;

    let platform_config = parse_repo_info(&remote_info.url)?;

    // Create platform service
    let platform = create_platform_service(&platform_config).await?;

    // Fetch from remote
    if !dry_run {
        println!("Fetching from {remote_name}...");
        workspace.git_fetch(&remote_name)?;
    }

    // Build change graph
    let graph = build_change_graph(&workspace)?;

    if graph.stacks.is_empty() {
        println!("No stacks to sync");
        return Ok(());
    }

    let default_branch = workspace.default_branch()?;
    let progress = CliProgress;

    // Sync each stack
    let mut total_pushed = 0;
    let mut total_created = 0;
    let mut total_updated = 0;

    for stack in &graph.stacks {
        if stack.segments.is_empty() {
            continue;
        }

        // Get the leaf bookmark (last segment)
        let leaf_bookmark = &stack.segments.last().unwrap().bookmarks[0].name;

        println!("Syncing stack: {leaf_bookmark}");

        let analysis = analyze_submission(&graph, leaf_bookmark)?;
        let plan = create_submission_plan(
            &analysis,
            platform.as_ref(),
            &remote_name,
            &default_branch,
        )
        .await?;

        let result = execute_submission(
            &plan,
            &mut workspace,
            platform.as_ref(),
            &progress,
            dry_run,
        )
        .await?;

        total_pushed += result.pushed_bookmarks.len();
        total_created += result.created_prs.len();
        total_updated += result.updated_prs.len();
    }

    // Summary
    println!();
    if dry_run {
        println!("Dry run complete");
    } else {
        println!(
            "Sync complete: {total_pushed} bookmarks pushed, {total_created} PRs created, {total_updated} PRs updated"
        );
    }

    Ok(())
}
