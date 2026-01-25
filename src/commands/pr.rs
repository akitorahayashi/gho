//! Pull request commands.

use crate::commands::account;
use crate::error::AppError;
use crate::github::GitHubClient;
use crate::models::PullRequestOutput;
use crate::storage::Storage;
use std::process::Command;

/// List open pull requests for a repository.
pub fn list(
    storage: &impl Storage,
    repo_spec: Option<&str>,
    limit: usize,
) -> Result<Vec<PullRequestOutput>, AppError> {
    let (_account, token) = account::get_active_with_token(storage)?;
    let client = GitHubClient::new(token)?;

    let (owner, repo) = match repo_spec {
        Some(spec) => parse_repo_spec(spec)?,
        None => detect_repo_from_git()?,
    };

    let prs = client.list_pull_requests(&owner, &repo, limit)?;

    let output: Vec<PullRequestOutput> = prs
        .into_iter()
        .map(|pr| PullRequestOutput {
            number: pr.number,
            title: pr.title,
            author: pr.user.login,
            branch: pr.head.branch,
            mergeable: pr.mergeable,
            actions_in_progress: false, // Would require additional API call
            ci_status: "unknown".to_string(), // Would require check runs API
        })
        .collect();

    Ok(output)
}

fn parse_repo_spec(spec: &str) -> Result<(String, String), AppError> {
    let parts: Vec<&str> = spec.split('/').collect();
    if parts.len() != 2 {
        return Err(AppError::invalid_input(format!(
            "invalid repository format '{}', expected owner/repo",
            spec
        )));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

fn detect_repo_from_git() -> Result<(String, String), AppError> {
    // Check GITHUB_REPOSITORY environment variable first
    if let Ok(repo) = std::env::var("GITHUB_REPOSITORY") {
        return parse_repo_spec(&repo);
    }

    // Try to get from git remote
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .map_err(|e| AppError::git(format!("failed to run git: {e}")))?;

    if !output.status.success() {
        return Err(AppError::git("no repository detected, provide owner/repo argument"));
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    parse_remote_url(&url)
}

fn parse_remote_url(url: &str) -> Result<(String, String), AppError> {
    // Handle SSH URLs: git@github.com:owner/repo.git
    if let Some(path) = url.strip_prefix("git@github.com:") {
        let path = path.trim_end_matches(".git");
        return parse_repo_spec(path);
    }

    // Handle HTTPS URLs: https://github.com/owner/repo.git
    if let Some(path) = url.strip_prefix("https://github.com/") {
        let path = path.trim_end_matches(".git");
        return parse_repo_spec(path);
    }

    Err(AppError::git(format!("unrecognized remote URL format: {url}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_remote_url_ssh() {
        let (owner, repo) = parse_remote_url("git@github.com:octocat/hello-world.git").unwrap();
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "hello-world");
    }

    #[test]
    fn parse_remote_url_https() {
        let (owner, repo) = parse_remote_url("https://github.com/octocat/hello-world.git").unwrap();
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "hello-world");
    }

    #[test]
    fn parse_remote_url_https_no_git_suffix() {
        let (owner, repo) = parse_remote_url("https://github.com/octocat/hello-world").unwrap();
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "hello-world");
    }
}
