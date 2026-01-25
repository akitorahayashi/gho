//! Account management commands.

use crate::error::AppError;
use crate::keychain;
use crate::models::{Account, AccountKind, AccountsFile, Protocol};
use crate::storage::Storage;

/// Add a new account.
#[allow(clippy::too_many_arguments)]
pub fn add(
    storage: &impl Storage,
    id: &str,
    username: &str,
    kind: AccountKind,
    token: &str,
    default_org: Option<String>,
    protocol: Protocol,
    clone_dir: Option<String>,
) -> Result<(), AppError> {
    let mut accounts = storage.load_accounts()?;

    // Check for duplicate ID
    if accounts.find_account(id).is_some() {
        return Err(AppError::invalid_input(format!("account '{id}' already exists")));
    }

    let account = Account {
        id: id.to_string(),
        kind,
        username: username.to_string(),
        default_org,
        protocol,
        clone_dir,
    };

    // Store token in keychain
    keychain::store_token(id, token)?;

    // Add account
    accounts.add_account(account);

    // If this is the first account, make it active
    if accounts.active_account_id.is_none() {
        accounts.active_account_id = Some(id.to_string());
    }

    // Save accounts, rolling back keychain on failure
    if let Err(e) = storage.save_accounts(&accounts) {
        // Attempt to clean up the keychain entry
        let _ = keychain::delete_token(id);
        return Err(e);
    }
    Ok(())
}

/// List all accounts.
pub fn list(storage: &impl Storage) -> Result<AccountsFile, AppError> {
    storage.load_accounts()
}

/// Switch the active account.
pub fn switch(storage: &impl Storage, id: &str) -> Result<(), AppError> {
    let mut accounts = storage.load_accounts()?;

    if accounts.find_account(id).is_none() {
        return Err(AppError::AccountNotFound(id.to_string()));
    }

    accounts.active_account_id = Some(id.to_string());
    storage.save_accounts(&accounts)?;
    Ok(())
}

/// Switch account interactively.
pub fn switch_interactive(storage: &impl Storage) -> Result<String, AppError> {
    if !atty::is(atty::Stream::Stdin) {
        return Err(AppError::TtyRequired);
    }

    let accounts = storage.load_accounts()?;
    let all_accounts = accounts.all_accounts();

    if all_accounts.is_empty() {
        return Err(AppError::config("no accounts configured"));
    }

    let options: Vec<String> = all_accounts
        .iter()
        .map(|a| {
            let active = accounts.active_account_id.as_deref() == Some(&a.id);
            let marker = if active { " (active)" } else { "" };
            format!("{} ({}){}", a.id, a.username, marker)
        })
        .collect();

    let selection = inquire::Select::new("Select account:", options)
        .prompt()
        .map_err(|e| AppError::config(format!("selection cancelled: {e}")))?;

    // Extract the account ID from the selection
    let selected_id = selection.split('(').next().map(|s| s.trim()).unwrap_or("");
    if selected_id.is_empty() {
        return Err(AppError::config(format!("could not parse selection: {}", selection)));
    }
    switch(storage, selected_id)?;
    Ok(selected_id.to_string())
}

/// Show the active account.
pub fn show(storage: &impl Storage) -> Result<Account, AppError> {
    let accounts = storage.load_accounts()?;
    accounts.active_account().cloned().ok_or(AppError::NoActiveAccount)
}

/// Remove an account.
pub fn remove(storage: &impl Storage, id: &str) -> Result<(), AppError> {
    let mut accounts = storage.load_accounts()?;

    if accounts.remove_account(id).is_none() {
        return Err(AppError::AccountNotFound(id.to_string()));
    }

    // Delete token from keychain (ignore errors if not found)
    let _ = keychain::delete_token(id);

    storage.save_accounts(&accounts)?;
    Ok(())
}

/// Get the active account with its token.
pub fn get_active_with_token(storage: &impl Storage) -> Result<(Account, String), AppError> {
    let account = show(storage)?;
    let token = keychain::get_token(&account.id)?;
    Ok((account, token))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::StateFile;
    use std::cell::RefCell;

    #[derive(Default)]
    struct MockStorage {
        accounts: RefCell<AccountsFile>,
    }

    impl Storage for MockStorage {
        fn load_accounts(&self) -> Result<AccountsFile, AppError> {
            Ok(self.accounts.borrow().clone())
        }

        fn save_accounts(&self, accounts: &AccountsFile) -> Result<(), AppError> {
            *self.accounts.borrow_mut() = accounts.clone();
            Ok(())
        }

        fn load_state(&self) -> Result<StateFile, AppError> {
            Ok(StateFile::default())
        }

        fn save_state(&self, _state: &StateFile) -> Result<(), AppError> {
            Ok(())
        }
    }

    #[test]
    fn list_returns_empty_initially() {
        let storage = MockStorage::default();
        let accounts = list(&storage).expect("should succeed");
        assert!(accounts.personal.is_empty());
        assert!(accounts.work.is_empty());
    }

    #[test]
    fn switch_to_nonexistent_account_fails() {
        let storage = MockStorage::default();
        let result = switch(&storage, "nonexistent");
        assert!(matches!(result, Err(AppError::AccountNotFound(_))));
    }

    #[test]
    fn show_without_active_fails() {
        let storage = MockStorage::default();
        let result = show(&storage);
        assert!(matches!(result, Err(AppError::NoActiveAccount)));
    }
}
