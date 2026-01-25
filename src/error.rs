//! Error types for gho.

use thiserror::Error;

/// Application-wide error type.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("No active account configured")]
    NoActiveAccount,

    #[error("Keychain error: {0}")]
    Keychain(String),

    #[error("GitHub API error: {0}")]
    GitHubApi(String),

    #[error("Git error: {0}")]
    Git(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("TTY required for interactive selection")]
    TtyRequired,

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl AppError {
    pub fn config<S: Into<String>>(msg: S) -> Self {
        AppError::Config(msg.into())
    }

    pub fn keychain<S: Into<String>>(msg: S) -> Self {
        AppError::Keychain(msg.into())
    }

    pub fn github_api<S: Into<String>>(msg: S) -> Self {
        AppError::GitHubApi(msg.into())
    }

    pub fn git<S: Into<String>>(msg: S) -> Self {
        AppError::Git(msg.into())
    }

    pub fn network<S: Into<String>>(msg: S) -> Self {
        AppError::Network(msg.into())
    }

    pub fn invalid_input<S: Into<String>>(msg: S) -> Self {
        AppError::InvalidInput(msg.into())
    }
}
