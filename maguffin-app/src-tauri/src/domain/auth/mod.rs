//! Authentication domain types for GitHub OAuth.
//!
//! This module handles GitHub OAuth device flow authentication and token management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Current authentication state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthState {
    /// User is not authenticated
    #[default]
    Unauthenticated,

    /// OAuth device flow is in progress
    Pending(DeviceFlowPending),

    /// User is authenticated
    Authenticated(AuthenticatedUser),
}

/// Device flow pending state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFlowPending {
    /// Code to show to user
    pub user_code: String,

    /// URL where user should enter the code
    pub verification_uri: String,

    /// Device code (internal use for polling)
    #[serde(skip_serializing)]
    pub device_code: String,

    /// When the code expires
    pub expires_at: DateTime<Utc>,

    /// Polling interval in seconds
    pub interval: u64,
}

/// Authenticated user information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    /// GitHub username
    pub login: String,

    /// GitHub user ID
    pub id: i64,

    /// User's display name
    pub name: Option<String>,

    /// User's email (if public)
    pub email: Option<String>,

    /// Avatar URL
    pub avatar_url: String,

    /// When authentication was completed
    pub authenticated_at: DateTime<Utc>,
}

/// OAuth token (stored securely in keyring).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    /// Access token
    pub access_token: String,

    /// Token type (usually "bearer")
    pub token_type: String,

    /// Token scopes
    pub scope: String,
}

/// Response from GitHub device flow initiation.
#[derive(Debug, Clone, Deserialize)]
pub struct DeviceCodeResponse {
    /// Device code for polling
    pub device_code: String,

    /// User code to display
    pub user_code: String,

    /// URL for user to enter code
    pub verification_uri: String,

    /// Seconds until code expires
    pub expires_in: u64,

    /// Polling interval in seconds
    pub interval: u64,
}

/// Response from GitHub device flow token poll.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum TokenPollResponse {
    /// Token received successfully
    Success(TokenResponse),

    /// Error (user hasn't authorized yet, etc.)
    Error(TokenPollError),
}

/// Successful token response.
#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    /// Access token
    pub access_token: String,

    /// Token type
    pub token_type: String,

    /// Token scope
    pub scope: String,
}

/// Error during token polling.
#[derive(Debug, Clone, Deserialize)]
pub struct TokenPollError {
    /// Error code
    pub error: String,

    /// Error description
    pub error_description: Option<String>,
}

impl TokenPollError {
    /// Check if user authorization is still pending.
    pub fn is_pending(&self) -> bool {
        self.error == "authorization_pending"
    }

    /// Check if the code has expired.
    pub fn is_expired(&self) -> bool {
        self.error == "expired_token"
    }

    /// Check if we're polling too fast.
    pub fn is_slow_down(&self) -> bool {
        self.error == "slow_down"
    }

    /// Check if the user denied authorization.
    pub fn is_access_denied(&self) -> bool {
        self.error == "access_denied"
    }
}

/// Current user response from GitHub API.
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubUser {
    /// GitHub username
    pub login: String,

    /// User ID
    pub id: i64,

    /// Display name
    pub name: Option<String>,

    /// Email (if public)
    pub email: Option<String>,

    /// Avatar URL
    pub avatar_url: String,
}

impl From<(GitHubUser, DateTime<Utc>)> for AuthenticatedUser {
    fn from((user, authenticated_at): (GitHubUser, DateTime<Utc>)) -> Self {
        Self {
            login: user.login,
            id: user.id,
            name: user.name,
            email: user.email,
            avatar_url: user.avatar_url,
            authenticated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_state_default() {
        let state = AuthState::default();
        assert!(matches!(state, AuthState::Unauthenticated));
    }

    #[test]
    fn test_token_poll_error_is_pending() {
        let error = TokenPollError {
            error: "authorization_pending".to_string(),
            error_description: None,
        };
        assert!(error.is_pending());
        assert!(!error.is_expired());
    }

    #[test]
    fn test_token_poll_error_is_slow_down() {
        let error = TokenPollError {
            error: "slow_down".to_string(),
            error_description: None,
        };
        assert!(error.is_slow_down());
    }

    #[test]
    fn test_auth_state_serialization() {
        let state = AuthState::Unauthenticated;
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("unauthenticated"));
    }
}
