//! Data models for gho.

use serde::{Deserialize, Serialize};

/// Git protocol for cloning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    #[default]
    Ssh,
    Https,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Ssh => write!(f, "ssh"),
            Protocol::Https => write!(f, "https"),
        }
    }
}

/// Account kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountKind {
    Personal,
    Work,
}

impl std::fmt::Display for AccountKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountKind::Personal => write!(f, "personal"),
            AccountKind::Work => write!(f, "work"),
        }
    }
}

/// A GitHub account configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Unique identifier for this account.
    pub id: String,
    /// Account kind (personal or work).
    pub kind: AccountKind,
    /// GitHub username.
    pub username: String,
    /// Default organization for operations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_org: Option<String>,
    /// Preferred protocol for cloning.
    #[serde(default)]
    pub protocol: Protocol,
    /// Directory for cloning repositories.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_dir: Option<String>,
}

/// Container for all accounts.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountsFile {
    /// Personal accounts.
    #[serde(default)]
    pub personal: Vec<Account>,
    /// Work accounts.
    #[serde(default)]
    pub work: Vec<Account>,
    /// Currently active account ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_account_id: Option<String>,
}

impl AccountsFile {
    /// Get all accounts as a flat list.
    pub fn all_accounts(&self) -> Vec<&Account> {
        self.personal.iter().chain(self.work.iter()).collect()
    }

    /// Find an account by ID.
    pub fn find_account(&self, id: &str) -> Option<&Account> {
        self.all_accounts().into_iter().find(|a| a.id == id)
    }

    /// Find an account by ID (mutable).
    pub fn find_account_mut(&mut self, id: &str) -> Option<&mut Account> {
        self.personal.iter_mut().chain(self.work.iter_mut()).find(|a| a.id == id)
    }

    /// Get the active account.
    pub fn active_account(&self) -> Option<&Account> {
        self.active_account_id.as_ref().and_then(|id| self.find_account(id))
    }

    /// Add an account.
    pub fn add_account(&mut self, account: Account) {
        match account.kind {
            AccountKind::Personal => self.personal.push(account),
            AccountKind::Work => self.work.push(account),
        }
    }

    /// Remove an account by ID.
    pub fn remove_account(&mut self, id: &str) -> Option<Account> {
        if let Some(pos) = self.personal.iter().position(|a| a.id == id) {
            if self.active_account_id.as_deref() == Some(id) {
                self.active_account_id = None;
            }
            return Some(self.personal.remove(pos));
        }
        if let Some(pos) = self.work.iter().position(|a| a.id == id) {
            if self.active_account_id.as_deref() == Some(id) {
                self.active_account_id = None;
            }
            return Some(self.work.remove(pos));
        }
        None
    }
}

/// Application state for gho.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StateFile {
    /// Last used organization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_org: Option<String>,
    /// Last used repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_repo: Option<String>,
}

/// Repository information from GitHub API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub ssh_url: String,
    pub clone_url: String,
    #[serde(default)]
    pub pushed_at: Option<String>,
    pub owner: RepositoryOwner,
}

/// Repository owner information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryOwner {
    pub login: String,
}

/// Pull request information from GitHub API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub user: PullRequestUser,
    pub head: PullRequestHead,
    #[serde(default)]
    pub mergeable: Option<bool>,
}

/// Pull request author.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestUser {
    pub login: String,
}

/// Pull request head branch info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestHead {
    #[serde(rename = "ref")]
    pub branch: String,
}

/// Output format for PR list.
#[derive(Debug, Clone, Serialize)]
pub struct PullRequestOutput {
    pub number: u64,
    pub title: String,
    pub author: String,
    pub branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mergeable: Option<bool>,
    pub actions_in_progress: bool,
    pub ci_status: String,
}
