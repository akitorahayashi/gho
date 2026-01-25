//! Storage layer for gho configuration and state.

use crate::config::Config;
use crate::error::AppError;
use crate::models::{AccountsFile, StateFile};
use std::fs;
use std::path::PathBuf;

/// Storage abstraction for accounts and state.
pub trait Storage {
    fn load_accounts(&self) -> Result<AccountsFile, AppError>;
    fn save_accounts(&self, accounts: &AccountsFile) -> Result<(), AppError>;
    fn load_state(&self) -> Result<StateFile, AppError>;
    fn save_state(&self, state: &StateFile) -> Result<(), AppError>;
}

/// Filesystem-based storage implementation.
#[derive(Debug, Clone)]
pub struct FilesystemStorage {
    config: Config,
}

impl FilesystemStorage {
    /// Create a new storage with the given configuration.
    pub fn new(config: &Config) -> Self {
        Self { config: config.clone() }
    }

    /// Create storage with default configuration.
    pub fn new_default() -> Result<Self, AppError> {
        let config = Config::new_default()?;
        Ok(Self::new(&config))
    }

    fn ensure_config_dir(&self) -> Result<(), AppError> {
        fs::create_dir_all(&self.config.config_path)?;
        Ok(())
    }

    fn accounts_path(&self) -> PathBuf {
        self.config.accounts_path()
    }

    fn state_path(&self) -> PathBuf {
        self.config.state_path()
    }
}

impl Storage for FilesystemStorage {
    fn load_accounts(&self) -> Result<AccountsFile, AppError> {
        let path = self.accounts_path();
        if !path.exists() {
            return Ok(AccountsFile::default());
        }
        let content = fs::read_to_string(&path)?;
        let accounts: AccountsFile = serde_json::from_str(&content)?;
        Ok(accounts)
    }

    fn save_accounts(&self, accounts: &AccountsFile) -> Result<(), AppError> {
        self.ensure_config_dir()?;
        let content = serde_json::to_string_pretty(accounts)?;
        fs::write(self.accounts_path(), content)?;
        Ok(())
    }

    fn load_state(&self) -> Result<StateFile, AppError> {
        let path = self.state_path();
        if !path.exists() {
            return Ok(StateFile::default());
        }
        let content = fs::read_to_string(&path)?;
        let state: StateFile = serde_json::from_str(&content)?;
        Ok(state)
    }

    fn save_state(&self, state: &StateFile) -> Result<(), AppError> {
        self.ensure_config_dir()?;
        let content = serde_json::to_string_pretty(state)?;
        fs::write(self.state_path(), content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Account, AccountKind, Protocol};
    use tempfile::TempDir;

    fn test_storage() -> (TempDir, FilesystemStorage) {
        let tmp = TempDir::new().expect("failed to create temp dir");
        let config = Config::with_path(tmp.path().join(".config").join("gho"));
        let storage = FilesystemStorage::new(&config);
        (tmp, storage)
    }

    #[test]
    fn load_accounts_returns_empty_when_no_file() {
        let (_tmp, storage) = test_storage();
        let accounts = storage.load_accounts().expect("should succeed");
        assert!(accounts.personal.is_empty());
        assert!(accounts.work.is_empty());
        assert!(accounts.active_account_id.is_none());
    }

    #[test]
    fn save_and_load_accounts() {
        let (_tmp, storage) = test_storage();
        let mut accounts = AccountsFile::default();
        accounts.add_account(Account {
            id: "test".to_string(),
            kind: AccountKind::Personal,
            username: "testuser".to_string(),
            default_org: None,
            protocol: Protocol::Ssh,
            clone_dir: None,
        });
        accounts.active_account_id = Some("test".to_string());

        storage.save_accounts(&accounts).expect("save should succeed");
        let loaded = storage.load_accounts().expect("load should succeed");

        assert_eq!(loaded.personal.len(), 1);
        assert_eq!(loaded.personal[0].id, "test");
        assert_eq!(loaded.active_account_id, Some("test".to_string()));
    }

    #[test]
    fn save_and_load_state() {
        let (_tmp, storage) = test_storage();
        let state = StateFile { last_org: Some("myorg".to_string()), last_repo: None };

        storage.save_state(&state).expect("save should succeed");
        let loaded = storage.load_state().expect("load should succeed");

        assert_eq!(loaded.last_org, Some("myorg".to_string()));
    }
}
