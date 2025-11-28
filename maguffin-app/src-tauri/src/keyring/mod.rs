//! Secure credential storage using the OS keyring.
//!
//! This module provides secure storage for GitHub tokens using
//! platform-native credential storage (Keychain, Credential Manager, Secret Service).

use crate::error::{AuthError, Result};

const SERVICE_NAME: &str = "maguffin-app";

/// Keyring-based credential storage.
pub struct KeyringStore {
    service: String,
}

impl Default for KeyringStore {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyringStore {
    /// Create a new keyring store.
    pub fn new() -> Self {
        Self {
            service: SERVICE_NAME.to_string(),
        }
    }

    /// Create a keyring store with a custom service name (useful for testing).
    pub fn with_service(service: String) -> Self {
        Self { service }
    }

    /// Store a GitHub token.
    pub fn store_token(&self, username: &str, token: &str) -> Result<()> {
        let entry = keyring::Entry::new(&self.service, username)
            .map_err(|e| AuthError::Keyring(e.to_string()))?;

        entry
            .set_password(token)
            .map_err(|e| AuthError::Keyring(e.to_string()))?;

        Ok(())
    }

    /// Retrieve a stored token.
    pub fn get_token(&self, username: &str) -> Result<Option<String>> {
        let entry = keyring::Entry::new(&self.service, username)
            .map_err(|e| AuthError::Keyring(e.to_string()))?;

        match entry.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AuthError::Keyring(e.to_string()).into()),
        }
    }

    /// Delete a stored token.
    pub fn delete_token(&self, username: &str) -> Result<()> {
        let entry = keyring::Entry::new(&self.service, username)
            .map_err(|e| AuthError::Keyring(e.to_string()))?;

        match entry.delete_password() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
            Err(e) => Err(AuthError::Keyring(e.to_string()).into()),
        }
    }

    /// Check if a token exists.
    pub fn has_token(&self, username: &str) -> Result<bool> {
        Ok(self.get_token(username)?.is_some())
    }
}

// Note: Keyring tests require actual keyring access which may not be available
// in all CI environments. These tests are marked as ignored by default.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyring_store_creation() {
        let store = KeyringStore::new();
        assert_eq!(store.service, SERVICE_NAME);
    }

    #[test]
    fn test_keyring_store_custom_service() {
        let store = KeyringStore::with_service("test-service".to_string());
        assert_eq!(store.service, "test-service");
    }

    // This test requires actual keyring access
    #[test]
    #[ignore]
    fn test_keyring_store_roundtrip() {
        let store = KeyringStore::with_service("maguffin-test".to_string());
        let username = "test-user";
        let token = "test-token-12345";

        // Store
        store.store_token(username, token).unwrap();

        // Retrieve
        let retrieved = store.get_token(username).unwrap();
        assert_eq!(retrieved, Some(token.to_string()));

        // Clean up
        store.delete_token(username).unwrap();

        // Verify deleted
        let deleted = store.get_token(username).unwrap();
        assert!(deleted.is_none());
    }
}
