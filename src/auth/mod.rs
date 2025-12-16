//! Authentication for GitHub and GitLab
//!
//! Supports CLI-based auth (gh, glab) and environment variables.

mod github;
mod gitlab;

pub use github::{get_github_auth, test_github_auth, GitHubAuthConfig};
pub use gitlab::{get_gitlab_auth, test_gitlab_auth, GitLabAuthConfig};

/// Source of authentication token
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthSource {
    /// Token from CLI tool (gh or glab)
    Cli,
    /// Token from environment variable
    EnvVar,
}
