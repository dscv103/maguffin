//! Provider abstraction layer for Git hosting services.
//!
//! This module provides traits that abstract the interface to Git hosting providers,
//! enabling future support for multiple providers (GitHub, GitLab, Bitbucket, etc.)
//! while maintaining a consistent internal API.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::auth::{AuthState, DeviceFlowPending, TokenPollError};
use crate::domain::pr::{MergeMethod, PullRequest, PullRequestDetails};
use crate::error::Result;

/// Identifies a Git hosting provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    /// GitHub (github.com or GitHub Enterprise)
    GitHub,
    /// GitLab (gitlab.com or self-hosted)
    GitLab,
    /// Bitbucket (bitbucket.org or Bitbucket Server)
    Bitbucket,
    /// Azure DevOps
    AzureDevOps,
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::GitHub => write!(f, "GitHub"),
            ProviderType::GitLab => write!(f, "GitLab"),
            ProviderType::Bitbucket => write!(f, "Bitbucket"),
            ProviderType::AzureDevOps => write!(f, "Azure DevOps"),
        }
    }
}

/// Configuration for a provider instance.
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// The type of provider.
    pub provider_type: ProviderType,
    /// API endpoint URL.
    pub api_url: String,
    /// Web URL for the provider (for opening in browser).
    pub web_url: String,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider_type: ProviderType::GitHub,
            api_url: "https://api.github.com/graphql".to_string(),
            web_url: "https://github.com".to_string(),
        }
    }
}

/// Trait for authentication with a Git hosting provider.
///
/// Implementations should support the appropriate OAuth flow for the provider.
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Start the authentication flow.
    ///
    /// For OAuth device flow, this returns the device code and verification URL.
    async fn start_auth(&self) -> Result<DeviceFlowPending>;

    /// Poll for authentication completion.
    ///
    /// This should be called periodically until it returns Ok(AuthState::Authenticated)
    /// or an error other than pending/slow_down.
    async fn poll_auth(&self, device_code: &str) -> std::result::Result<AuthState, TokenPollError>;

    /// Restore authentication from stored credentials.
    ///
    /// Returns the authenticated state if valid credentials exist.
    async fn restore_auth(&self) -> Result<Option<AuthState>>;

    /// Log out and clear stored credentials.
    async fn logout(&self) -> Result<()>;

    /// Get the current authentication state.
    async fn auth_state(&self) -> AuthState;
}

/// Trait for pull request operations.
///
/// Provides methods for listing, viewing, creating, and managing pull requests.
#[async_trait]
pub trait PullRequestProvider: Send + Sync {
    /// List open pull requests for a repository.
    ///
    /// # Arguments
    /// * `owner` - Repository owner (user or organization)
    /// * `repo` - Repository name
    /// * `base_branch` - Optional base branch to filter by
    async fn list_pull_requests(
        &self,
        owner: &str,
        repo: &str,
        base_branch: Option<&str>,
    ) -> Result<Vec<PullRequest>>;

    /// Get detailed information for a specific pull request.
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `number` - Pull request number
    async fn get_pull_request(
        &self,
        owner: &str,
        repo: &str,
        number: i64,
    ) -> Result<PullRequestDetails>;

    /// Create a new pull request.
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `title` - Pull request title
    /// * `body` - Pull request description
    /// * `head` - Source branch name
    /// * `base` - Target branch name
    /// * `draft` - Whether to create as a draft
    ///
    /// # Returns
    /// The created pull request number.
    async fn create_pull_request(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        body: Option<&str>,
        head: &str,
        base: &str,
        draft: bool,
    ) -> Result<i64>;

    /// Merge a pull request.
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `pr_id` - Provider-specific pull request ID
    /// * `method` - Merge method to use
    async fn merge_pull_request(
        &self,
        owner: &str,
        repo: &str,
        pr_id: &str,
        method: MergeMethod,
    ) -> Result<bool>;

    /// Close a pull request without merging.
    async fn close_pull_request(&self, owner: &str, repo: &str, pr_id: &str) -> Result<bool>;

    /// Update a pull request's base branch.
    async fn update_pull_request_base(
        &self,
        owner: &str,
        repo: &str,
        pr_id: &str,
        new_base: &str,
    ) -> Result<bool>;
}

/// Trait for repository metadata operations.
#[async_trait]
pub trait RepositoryProvider: Send + Sync {
    /// Get the repository ID (provider-specific identifier).
    async fn get_repository_id(&self, owner: &str, repo: &str) -> Result<String>;

    /// Get the default branch name for a repository.
    async fn get_default_branch(&self, owner: &str, repo: &str) -> Result<String>;
}

/// Combined provider that implements all provider traits.
///
/// This is the main interface for interacting with a Git hosting provider.
pub trait Provider: AuthProvider + PullRequestProvider + RepositoryProvider {}

// Blanket implementation: any type that implements all traits is a Provider
impl<T: AuthProvider + PullRequestProvider + RepositoryProvider> Provider for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_display() {
        assert_eq!(ProviderType::GitHub.to_string(), "GitHub");
        assert_eq!(ProviderType::GitLab.to_string(), "GitLab");
        assert_eq!(ProviderType::Bitbucket.to_string(), "Bitbucket");
        assert_eq!(ProviderType::AzureDevOps.to_string(), "Azure DevOps");
    }

    #[test]
    fn test_provider_type_serialization() {
        let github = ProviderType::GitHub;
        let json = serde_json::to_string(&github).unwrap();
        assert_eq!(json, "\"github\"");

        let parsed: ProviderType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ProviderType::GitHub);
    }

    #[test]
    fn test_provider_config_default() {
        let config = ProviderConfig::default();
        assert_eq!(config.provider_type, ProviderType::GitHub);
        assert_eq!(config.api_url, "https://api.github.com/graphql");
        assert_eq!(config.web_url, "https://github.com");
    }
}
