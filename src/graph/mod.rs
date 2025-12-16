//! Change graph building
//!
//! Analyzes jj bookmarks to build a graph of stacked changes.

mod builder;

pub use builder::build_change_graph;
