//! Keychain integration for token storage.

use crate::error::AppError;
use keyring::Entry;

const SERVICE_NAME: &str = "gho";

/// Store a token in the macOS Keychain.
pub fn store_token(account_id: &str, token: &str) -> Result<(), AppError> {
    let entry = Entry::new(SERVICE_NAME, account_id)
        .map_err(|e| AppError::keychain(format!("failed to create keychain entry: {e}")))?;
    entry
        .set_password(token)
        .map_err(|e| AppError::keychain(format!("failed to store token: {e}")))?;
    Ok(())
}

/// Retrieve a token from the macOS Keychain.
pub fn get_token(account_id: &str) -> Result<String, AppError> {
    // Check for environment variable overrides first
    if let Ok(token) = std::env::var("GH_TOKEN") {
        return Ok(token);
    }
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        return Ok(token);
    }

    let entry = Entry::new(SERVICE_NAME, account_id)
        .map_err(|e| AppError::keychain(format!("failed to create keychain entry: {e}")))?;
    entry.get_password().map_err(|e| AppError::keychain(format!("failed to retrieve token: {e}")))
}

/// Delete a token from the macOS Keychain.
pub fn delete_token(account_id: &str) -> Result<(), AppError> {
    let entry = Entry::new(SERVICE_NAME, account_id)
        .map_err(|e| AppError::keychain(format!("failed to create keychain entry: {e}")))?;
    entry
        .delete_credential()
        .map_err(|e| AppError::keychain(format!("failed to delete token: {e}")))?;
    Ok(())
}

/// Mask a token for display.
pub fn mask_token(token: &str) -> String {
    if token.len() <= 8 {
        return "*".repeat(token.len());
    }
    format!("{}...{}", &token[..4], &token[token.len() - 4..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_token_hides_middle() {
        let masked = mask_token("ghp_1234567890abcdef");
        assert!(masked.starts_with("ghp_"));
        assert!(masked.contains("..."));
        assert!(masked.ends_with("cdef"));
    }

    #[test]
    fn mask_token_short_string() {
        let masked = mask_token("short");
        assert_eq!(masked, "*****");
    }
}
