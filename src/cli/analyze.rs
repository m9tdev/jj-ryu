//! Default analyze command - print stack graph visualization

use jj_ryu::error::Result;
use jj_ryu::graph::build_change_graph;
use jj_ryu::repo::JjWorkspace;
use std::path::Path;

/// Run the analyze command (default when no subcommand given)
///
/// Prints a text-based visualization of the bookmark stacks.
pub async fn run_analyze(path: &Path) -> Result<()> {
    // Open workspace
    let workspace = JjWorkspace::open(path)?;

    // Build change graph
    let graph = build_change_graph(&workspace)?;

    if graph.stacks.is_empty() {
        println!("No bookmark stacks found");
        println!();
        println!("Stacks are bookmarks that point to commits between trunk and your work.");
        println!("Create a bookmark with: jj bookmark create <name>");
        return Ok(());
    }

    // Print stacks
    println!("Bookmark Stacks");
    println!("===============");
    println!();

    for (i, stack) in graph.stacks.iter().enumerate() {
        if stack.segments.is_empty() {
            continue;
        }

        // Print stack header with leaf bookmark name
        let leaf = stack.segments.last().unwrap();
        let leaf_name = &leaf.bookmarks[0].name;
        println!("Stack #{}: {}", i + 1, leaf_name);
        println!();

        // Print each segment in reverse order (newest/leaf first, oldest last)
        for segment in stack.segments.iter().rev() {
            let bookmark_names: Vec<&str> =
                segment.bookmarks.iter().map(|b| b.name.as_str()).collect();

            // Print commits in segment (already newest-first from revset)
            for (j, change) in segment.changes.iter().enumerate() {
                let is_first_in_segment = j == 0;
                let commit_short = &change.commit_id[..8.min(change.commit_id.len())];
                let change_short = &change.change_id[..8.min(change.change_id.len())];

                let desc = if change.description_first_line.is_empty() {
                    "(no description)"
                } else {
                    &change.description_first_line
                };

                // Truncate description (char-safe for UTF-8)
                let max_desc = 50;
                let desc_display = if desc.chars().count() > max_desc {
                    format!("{}...", desc.chars().take(max_desc - 3).collect::<String>())
                } else {
                    desc.to_string()
                };

                let marker = if change.is_working_copy { "@" } else { "○" };

                // Show bookmark on first commit of segment (the tip)
                if is_first_in_segment && !bookmark_names.is_empty() {
                    for bm in &bookmark_names {
                        let bookmark = segment.bookmarks.iter().find(|b| b.name == *bm).unwrap();
                        let sync_status = if bookmark.is_synced {
                            " ✓"
                        } else if bookmark.has_remote {
                            " ↑"
                        } else {
                            ""
                        };
                        println!("       [{bm}]{sync_status}");
                    }
                }
                println!("    {marker}  {change_short} {commit_short} {desc_display}");
                println!("    │");
            }
        }

        // Print trunk base at bottom
        println!("  trunk()");
        println!();
    }

    // Summary
    let total_bookmarks: usize = graph.stacks.iter().map(|s| s.segments.len()).sum();
    println!(
        "{} stack{}, {} bookmark{}",
        graph.stacks.len(),
        if graph.stacks.len() == 1 { "" } else { "s" },
        total_bookmarks,
        if total_bookmarks == 1 { "" } else { "s" }
    );

    if graph.excluded_bookmark_count > 0 {
        println!(
            "({} bookmark{} excluded due to merge commits)",
            graph.excluded_bookmark_count,
            if graph.excluded_bookmark_count == 1 {
                ""
            } else {
                "s"
            }
        );
    }

    println!();
    println!("Legend: ✓ = synced with remote, ↑ = needs push, @ = working copy");
    println!();
    println!("To submit a stack: ryu submit <bookmark>");

    Ok(())
}
