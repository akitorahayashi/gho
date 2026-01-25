//! Application configuration for gho.

use crate::error::AppError;
use std::path::PathBuf;

/// Application-wide configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Base path for configuration files.
    pub config_path: PathBuf,
}

impl Config {
    /// Create a new configuration with custom config path.
    pub fn with_path(path: PathBuf) -> Self {
        Self { config_path: path }
    }

    /// Create configuration using the HOME-based config directory.
    ///
    /// Uses $HOME/.config/gho for consistency across platforms and tests.
    pub fn new_default() -> Result<Self, AppError> {
        let home = std::env::var("HOME")
            .map_err(|_| AppError::config("HOME environment variable not set"))?;
        let config_path = PathBuf::from(home).join(".config").join("gho");
        Ok(Self { config_path })
    }

    /// Path to the accounts file.
    pub fn accounts_path(&self) -> PathBuf {
        self.config_path.join("accounts.json")
    }

    /// Path to the state file.
    pub fn state_path(&self) -> PathBuf {
        self.config_path.join("state.json")
    }
}
