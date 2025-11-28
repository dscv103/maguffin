//! Authentication Service.
//!
//! This module provides the OAuth device flow implementation for GitHub authentication.

use crate::domain::auth::{
    AuthState, AuthenticatedUser, DeviceCodeResponse, DeviceFlowPending, GitHubUser,
    TokenPollError, TokenResponse,
};
use crate::error::{AuthError, Result};
use crate::keyring::KeyringStore;
use chrono::{Duration, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;

/// GitHub OAuth client ID for device flow.
/// This should be configured for your application.
const GITHUB_CLIENT_ID: &str = "Ov23liYwNsRRRrKOQCvj";

/// GitHub device flow endpoints.
const DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const USER_API_URL: &str = "https://api.github.com/user";

/// Service for handling GitHub authentication.
pub struct AuthService {
    /// HTTP client
    http: reqwest::Client,

    /// Current authentication state
    state: Arc<RwLock<AuthState>>,

    /// Keyring for secure token storage
    keyring: KeyringStore,

    /// Device code for polling (only set during pending state)
    device_code: Arc<RwLock<Option<String>>>,
}

impl AuthService {
    /// Create a new authentication service.
    pub fn new() -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent("maguffin-app/0.1.0")
            .build()
            .map_err(|e| AuthError::OAuthFailed(e.to_string()))?;

        Ok(Self {
            http,
            state: Arc::new(RwLock::new(AuthState::Unauthenticated)),
            keyring: KeyringStore::new(),
            device_code: Arc::new(RwLock::new(None)),
        })
    }

    /// Get the current authentication state.
    pub async fn get_state(&self) -> AuthState {
        self.state.read().await.clone()
    }

    /// Try to restore authentication from stored credentials.
    pub async fn try_restore(&self) -> Result<AuthState> {
        // Check if we have a stored token
        if let Some(token) = self.keyring.get_token("github")? {
            // Verify the token is still valid by fetching user info
            match self.fetch_user(&token).await {
                Ok(user) => {
                    let authenticated_user = AuthenticatedUser::from((user, Utc::now()));
                    let state = AuthState::Authenticated(authenticated_user);
                    *self.state.write().await = state.clone();
                    return Ok(state);
                }
                Err(_) => {
                    // Token is invalid, clear it
                    let _ = self.keyring.delete_token("github");
                }
            }
        }

        Ok(AuthState::Unauthenticated)
    }

    /// Start the GitHub device flow.
    pub async fn start_device_flow(&self) -> Result<AuthState> {
        let response = self
            .http
            .post(DEVICE_CODE_URL)
            .header("Accept", "application/json")
            .form(&[("client_id", GITHUB_CLIENT_ID), ("scope", "repo")])
            .send()
            .await
            .map_err(|e| AuthError::OAuthFailed(e.to_string()))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::OAuthFailed(format!("Device flow failed: {}", text)).into());
        }

        let device_response: DeviceCodeResponse = response
            .json()
            .await
            .map_err(|e| AuthError::OAuthFailed(e.to_string()))?;

        // Store the device code for polling
        *self.device_code.write().await = Some(device_response.device_code.clone());

        let pending = DeviceFlowPending {
            user_code: device_response.user_code,
            verification_uri: device_response.verification_uri,
            device_code: device_response.device_code,
            expires_at: Utc::now() + Duration::seconds(device_response.expires_in as i64),
            interval: device_response.interval,
        };

        let state = AuthState::Pending(pending);
        *self.state.write().await = state.clone();

        Ok(state)
    }

    /// Poll the device flow for completion.
    pub async fn poll_device_flow(&self) -> Result<AuthState> {
        let device_code = {
            let code = self.device_code.read().await;
            code.clone()
                .ok_or_else(|| AuthError::OAuthFailed("No device flow in progress".to_string()))?
        };

        let response = self
            .http
            .post(TOKEN_URL)
            .header("Accept", "application/json")
            .form(&[
                ("client_id", GITHUB_CLIENT_ID),
                ("device_code", &device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await
            .map_err(|e| AuthError::OAuthFailed(e.to_string()))?;

        let text = response
            .text()
            .await
            .map_err(|e| AuthError::OAuthFailed(e.to_string()))?;

        // Try to parse as success first
        if let Ok(token_response) = serde_json::from_str::<TokenResponse>(&text) {
            return self.complete_auth(token_response).await;
        }

        // Try to parse as error
        if let Ok(error) = serde_json::from_str::<TokenPollError>(&text) {
            if error.is_pending() {
                // Still waiting for user
                return Ok(self.state.read().await.clone());
            } else if error.is_slow_down() {
                // Need to slow down polling
                return Ok(self.state.read().await.clone());
            } else if error.is_expired() {
                *self.state.write().await = AuthState::Unauthenticated;
                *self.device_code.write().await = None;
                return Err(AuthError::OAuthFailed("Device code expired".to_string()).into());
            } else if error.is_access_denied() {
                *self.state.write().await = AuthState::Unauthenticated;
                *self.device_code.write().await = None;
                return Err(AuthError::OAuthFailed("Access denied by user".to_string()).into());
            }
        }

        Err(AuthError::OAuthFailed(format!("Unexpected response: {}", text)).into())
    }

    /// Complete authentication with a token.
    async fn complete_auth(&self, token_response: TokenResponse) -> Result<AuthState> {
        let token = &token_response.access_token;

        // Fetch user info
        let user = self.fetch_user(token).await?;

        // Store the token
        self.keyring.store_token("github", token)?;

        // Clear device code
        *self.device_code.write().await = None;

        // Update state
        let authenticated_user = AuthenticatedUser::from((user, Utc::now()));
        let state = AuthState::Authenticated(authenticated_user);
        *self.state.write().await = state.clone();

        Ok(state)
    }

    /// Fetch user information from GitHub API.
    async fn fetch_user(&self, token: &str) -> Result<GitHubUser> {
        let response = self
            .http
            .get(USER_API_URL)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| AuthError::OAuthFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthError::TokenExpired.into());
        }

        let user: GitHubUser = response
            .json()
            .await
            .map_err(|e| AuthError::OAuthFailed(e.to_string()))?;

        Ok(user)
    }

    /// Get the stored access token.
    pub fn get_token(&self) -> Result<Option<String>> {
        self.keyring.get_token("github")
    }

    /// Log out and clear stored credentials.
    pub async fn logout(&self) -> Result<()> {
        self.keyring.delete_token("github")?;
        *self.state.write().await = AuthState::Unauthenticated;
        *self.device_code.write().await = None;
        Ok(())
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new().expect("Failed to create AuthService")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_service_creation() {
        let service = AuthService::new();
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_initial_state_is_unauthenticated() {
        let service = AuthService::new().unwrap();
        let state = service.get_state().await;
        assert!(matches!(state, AuthState::Unauthenticated));
    }
}
