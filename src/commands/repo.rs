//! Repository management commands.

use crate::commands::account;
use crate::error::AppError;
use crate::github::GitHubClient;
use crate::models::{Protocol, Repository};
use crate::storage::Storage;
use std::path::Path;
use std::process::Command;

/// List repositories for the active account.
pub fn list(
    storage: &impl Storage,
    org: Option<&str>,
    limit: usize,
) -> Result<Vec<Repository>, AppError> {
    let (account, token) = account::get_active_with_token(storage)?;
    let client = GitHubClient::new(token)?;

    let repos = match org.or(account.default_org.as_deref()) {
        Some(org) => client.list_org_repos(org, limit)?,
        None => client.list_user_repos(&account.username, limit)?,
    };

    Ok(repos)
}

/// Clone a repository.
pub fn clone(storage: &impl Storage, repo_spec: &str) -> Result<(), AppError> {
    let (account, _token) = account::get_active_with_token(storage)?;

    let (owner, repo) = parse_repo_spec(repo_spec)?;
    let clone_url = build_clone_url(&owner, repo, account.protocol);

    let target_dir = match &account.clone_dir {
        Some(dir) => Path::new(dir).join(repo),
        None => Path::new(repo).to_path_buf(),
    };

    if target_dir.exists() {
        return Err(AppError::git(format!("directory '{}' already exists", target_dir.display())));
    }

    let status = Command::new("git")
        .arg("clone")
        .arg(&clone_url)
        .arg(&target_dir)
        .status()
        .map_err(|e| AppError::git(format!("failed to run git: {e}")))?;

    if !status.success() {
        return Err(AppError::git(format!("git clone failed with status {status}")));
    }

    Ok(())
}

/// Bulk clone repositories from an organization.
pub fn clone_org(storage: &impl Storage, org: &str, limit: usize) -> Result<Vec<String>, AppError> {
    let (account, token) = account::get_active_with_token(storage)?;
    let client = GitHubClient::new(token)?;

    let repos = client.list_org_repos(org, limit)?;
    let mut cloned = Vec::new();

    for repo in repos {
        let clone_url = match account.protocol {
            Protocol::Ssh => &repo.ssh_url,
            Protocol::Https => &repo.clone_url,
        };

        let target_dir = match &account.clone_dir {
            Some(dir) => Path::new(dir).join(&repo.name),
            None => Path::new(&repo.name).to_path_buf(),
        };

        if target_dir.exists() {
            eprintln!("⏭️  Skipping {} (already exists)", repo.name);
            continue;
        }

        let status = Command::new("git")
            .arg("clone")
            .arg(clone_url)
            .arg(&target_dir)
            .status()
            .map_err(|e| AppError::git(format!("failed to run git: {e}")))?;

        if status.success() {
            cloned.push(repo.name);
        } else {
            eprintln!("⚠️  Failed to clone {}", repo.name);
        }
    }

    Ok(cloned)
}

fn parse_repo_spec(spec: &str) -> Result<(String, &str), AppError> {
    let parts: Vec<&str> = spec.split('/').collect();
    if parts.len() != 2 {
        return Err(AppError::invalid_input(format!(
            "invalid repository format '{}', expected owner/repo",
            spec
        )));
    }
    Ok((parts[0].to_string(), parts[1]))
}

fn build_clone_url(owner: &str, repo: &str, protocol: Protocol) -> String {
    match protocol {
        Protocol::Ssh => format!("git@github.com:{}/{}.git", owner, repo),
        Protocol::Https => format!("https://github.com/{}/{}.git", owner, repo),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_repo_spec_valid() {
        let (owner, repo) = parse_repo_spec("octocat/hello-world").unwrap();
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "hello-world");
    }

    #[test]
    fn parse_repo_spec_invalid() {
        let result = parse_repo_spec("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn build_clone_url_ssh() {
        let url = build_clone_url("octocat", "hello-world", Protocol::Ssh);
        assert_eq!(url, "git@github.com:octocat/hello-world.git");
    }

    #[test]
    fn build_clone_url_https() {
        let url = build_clone_url("octocat", "hello-world", Protocol::Https);
        assert_eq!(url, "https://github.com/octocat/hello-world.git");
    }
}
