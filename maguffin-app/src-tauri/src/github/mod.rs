//! GitHub GraphQL client module.
//!
//! This module provides the client for interacting with GitHub's GraphQL API.
//! It handles authentication, rate limiting, pagination, and query execution.

pub mod auth_service;
pub mod pr_service;
pub mod queries;
pub mod stack_service;

use crate::error::{GitHubError, Result};
use std::sync::Arc;
use tokio::sync::RwLock;

pub use auth_service::AuthService;
pub use pr_service::PrService;
pub use stack_service::StackService;

/// GitHub GraphQL API client.
pub struct GitHubClient {
    /// HTTP client
    http: reqwest::Client,

    /// Access token
    token: Arc<RwLock<Option<String>>>,

    /// API endpoint
    endpoint: String,
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self {
            http: reqwest::Client::new(),
            token: Arc::new(RwLock::new(None)),
            endpoint: "https://api.github.com/graphql".to_string(),
        }
    }
}

impl GitHubClient {
    /// Create a new GitHub client.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(endpoint: String) -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent("maguffin-app/0.1.0")
            .build()
            .map_err(|e| GitHubError::Http(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            http,
            token: Arc::new(RwLock::new(None)),
            endpoint,
        })
    }

    /// Set the access token.
    pub async fn set_token(&self, token: String) {
        let mut guard = self.token.write().await;
        *guard = Some(token);
    }

    /// Clear the access token.
    pub async fn clear_token(&self) {
        let mut guard = self.token.write().await;
        *guard = None;
    }

    /// Check if a token is set.
    pub async fn has_token(&self) -> bool {
        self.token.read().await.is_some()
    }

    /// Execute a GraphQL query.
    pub async fn query<T>(&self, query: &str, variables: serde_json::Value) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let token = self.token.read().await;
        let token = token
            .as_ref()
            .ok_or_else(|| GitHubError::Unauthorized("No token set".to_string()))?;

        let body = serde_json::json!({
            "query": query,
            "variables": variables,
        });

        let response = self
            .http
            .post(&self.endpoint)
            .bearer_auth(token)
            .json(&body)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(GitHubError::Unauthorized("Invalid token".to_string()).into());
        }

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(GitHubError::GraphQL(format!("{}: {}", status, text)).into());
        }

        #[derive(serde::Deserialize)]
        struct GraphQLResponse<D> {
            data: Option<D>,
            errors: Option<Vec<GraphQLError>>,
        }

        #[derive(serde::Deserialize)]
        struct GraphQLError {
            message: String,
        }

        let response: GraphQLResponse<T> = response.json().await?;

        if let Some(errors) = response.errors {
            let messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
            return Err(GitHubError::GraphQL(messages.join("; ")).into());
        }

        response
            .data
            .ok_or_else(|| GitHubError::GraphQL("Empty response".to_string()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = GitHubClient::new("https://api.github.com/graphql".to_string()).unwrap();
        assert!(!client.has_token().await);
    }

    #[tokio::test]
    async fn test_set_token() {
        let client = GitHubClient::new("https://api.github.com/graphql".to_string()).unwrap();
        client.set_token("test_token".to_string()).await;
        assert!(client.has_token().await);
    }

    #[tokio::test]
    async fn test_clear_token() {
        let client = GitHubClient::new("https://api.github.com/graphql".to_string()).unwrap();
        client.set_token("test_token".to_string()).await;
        client.clear_token().await;
        assert!(!client.has_token().await);
    }
}
