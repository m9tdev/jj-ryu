//! Auth command - test and manage authentication

use jj_ryu::auth::{get_github_auth, get_gitlab_auth, test_github_auth, test_gitlab_auth};
use jj_ryu::error::Result;
use jj_ryu::types::Platform;

/// Run the auth test command
pub async fn run_auth_test(platform: Platform) -> Result<()> {
    match platform {
        Platform::GitHub => {
            println!("Testing GitHub authentication...");
            let config = get_github_auth().await?;
            let username = test_github_auth(&config).await?;
            println!("Authenticated as: {username}");
            println!("Token source: {:?}", config.source);
        }
        Platform::GitLab => {
            println!("Testing GitLab authentication...");
            let config = get_gitlab_auth(None).await?;
            let username = test_gitlab_auth(&config).await?;
            println!("Authenticated as: {username}");
            println!("Token source: {:?}", config.source);
            println!("Host: {}", config.host);
        }
    }
    Ok(())
}

/// Run the auth setup command (show instructions)
pub fn run_auth_setup(platform: Platform) {
    match platform {
        Platform::GitHub => {
            println!("GitHub Authentication Setup");
            println!("===========================");
            println!();
            println!("Option 1: GitHub CLI (recommended)");
            println!("  Install: https://cli.github.com/");
            println!("  Run: gh auth login");
            println!();
            println!("Option 2: Environment variable");
            println!("  Set GITHUB_TOKEN or GH_TOKEN");
            println!();
            println!("For GitHub Enterprise:");
            println!("  Set GH_HOST to your instance hostname");
        }
        Platform::GitLab => {
            println!("GitLab Authentication Setup");
            println!("===========================");
            println!();
            println!("Option 1: GitLab CLI (glab)");
            println!("  Install: https://gitlab.com/gitlab-org/cli");
            println!("  Run: glab auth login");
            println!();
            println!("Option 2: Environment variable");
            println!("  Set GITLAB_TOKEN or GL_TOKEN");
            println!();
            println!("For self-hosted GitLab:");
            println!("  Set GITLAB_HOST to your instance hostname");
        }
    }
}

/// Wrapper for auth commands
pub async fn run_auth(platform: Platform, action: &str) -> Result<()> {
    match action {
        "test" => run_auth_test(platform).await,
        "setup" => {
            run_auth_setup(platform);
            Ok(())
        }
        _ => {
            println!("Unknown action: {action}. Use 'test' or 'setup'.");
            Ok(())
        }
    }
}
