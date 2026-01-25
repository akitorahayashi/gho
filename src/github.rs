//! GitHub API client.

use crate::error::AppError;
use crate::models::{PullRequest, Repository};
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use std::time::Duration;

const GITHUB_API_BASE: &str = "https://api.github.com";
const DEFAULT_TIMEOUT_SECS: u64 = 30;
const DEFAULT_LIMIT: usize = 30;

/// GitHub API client.
pub struct GitHubClient {
    client: Client,
    token: String,
}

impl GitHubClient {
    /// Create a new GitHub client with the given token.
    pub fn new(token: String) -> Result<Self, AppError> {
        let client =
            Client::builder()
                .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
                .build()
                .map_err(|e| AppError::network(format!("failed to create HTTP client: {e}")))?;
        Ok(Self { client, token })
    }

    fn request(&self, url: &str) -> Result<reqwest::blocking::Response, AppError> {
        let response = self
            .client
            .get(url)
            .header(USER_AGENT, "gho")
            .header(AUTHORIZATION, format!("Bearer {}", self.token))
            .header(ACCEPT, "application/vnd.github+json")
            .send()
            .map_err(|e| AppError::network(format!("request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(AppError::github_api(format!("API error {status}: {body}")));
        }

        Ok(response)
    }

    /// List repositories for a user.
    pub fn list_user_repos(
        &self,
        username: &str,
        limit: usize,
    ) -> Result<Vec<Repository>, AppError> {
        let limit = if limit == 0 { DEFAULT_LIMIT } else { limit };
        let url = format!(
            "{}/users/{}/repos?sort=pushed&direction=desc&per_page={}",
            GITHUB_API_BASE, username, limit
        );
        let response = self.request(&url)?;
        let repos: Vec<Repository> = response
            .json()
            .map_err(|e| AppError::github_api(format!("failed to parse response: {e}")))?;
        Ok(repos)
    }

    /// List repositories for an organization.
    pub fn list_org_repos(&self, org: &str, limit: usize) -> Result<Vec<Repository>, AppError> {
        let limit = if limit == 0 { DEFAULT_LIMIT } else { limit };
        let url = format!(
            "{}/orgs/{}/repos?sort=pushed&direction=desc&per_page={}",
            GITHUB_API_BASE, org, limit
        );
        let response = self.request(&url)?;
        let repos: Vec<Repository> = response
            .json()
            .map_err(|e| AppError::github_api(format!("failed to parse response: {e}")))?;
        Ok(repos)
    }

    /// Get a specific repository.
    pub fn get_repo(&self, owner: &str, repo: &str) -> Result<Repository, AppError> {
        let url = format!("{}/repos/{}/{}", GITHUB_API_BASE, owner, repo);
        let response = self.request(&url)?;
        let repository: Repository = response
            .json()
            .map_err(|e| AppError::github_api(format!("failed to parse response: {e}")))?;
        Ok(repository)
    }

    /// List open pull requests for a repository.
    pub fn list_pull_requests(
        &self,
        owner: &str,
        repo: &str,
        limit: usize,
    ) -> Result<Vec<PullRequest>, AppError> {
        let limit = if limit == 0 { DEFAULT_LIMIT } else { limit };
        let url = format!(
            "{}/repos/{}/{}/pulls?state=open&sort=updated&direction=desc&per_page={}",
            GITHUB_API_BASE, owner, repo, limit
        );
        let response = self.request(&url)?;
        let prs: Vec<PullRequest> = response
            .json()
            .map_err(|e| AppError::github_api(format!("failed to parse response: {e}")))?;
        Ok(prs)
    }
}
