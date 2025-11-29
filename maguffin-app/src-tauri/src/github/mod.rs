//! GitHub GraphQL client module.
//!
//! This module provides the client for interacting with GitHub's GraphQL API.
//! It handles authentication, rate limiting, pagination, and query execution.

pub mod auth_service;
pub mod pr_service;
pub mod queries;
pub mod stack_service;
pub mod sync_service;

use crate::domain::sync::RateLimitInfo;
use crate::error::{GitHubError, Result};
use chrono::{DateTime, TimeZone, Utc};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;

pub use auth_service::AuthService;
pub use pr_service::PrService;
pub use stack_service::StackService;
pub use sync_service::SyncService;

/// Default backoff duration in seconds when rate limited
const DEFAULT_BACKOFF_SECS: u64 = 60;

/// Maximum number of retries for rate limit backoff
const MAX_RETRIES: u32 = 3;

/// Rate limit state for tracking GitHub API limits.
#[derive(Debug, Clone, Default)]
pub struct RateLimitState {
    /// Current rate limit info
    pub info: Option<RateLimitInfo>,
    /// Number of consecutive rate limit hits
    pub consecutive_hits: u32,
}

impl RateLimitState {
    /// Update rate limit info from response headers.
    pub fn update_from_headers(&mut self, remaining: u32, limit: u32, reset_timestamp: i64) {
        let resets_at = Utc.timestamp_opt(reset_timestamp, 0).single()
            .unwrap_or_else(Utc::now);
        
        self.info = Some(RateLimitInfo {
            remaining,
            limit,
            resets_at,
        });
        
        // Reset consecutive hits if we have remaining quota
        if remaining > 0 {
            self.consecutive_hits = 0;
        }
    }

    /// Mark that we hit a rate limit.
    pub fn mark_rate_limited(&mut self, reset_timestamp: Option<i64>) {
        self.consecutive_hits += 1;
        
        let resets_at = reset_timestamp
            .and_then(|ts| Utc.timestamp_opt(ts, 0).single())
            .unwrap_or_else(|| Utc::now() + chrono::Duration::minutes(15));
        
        self.info = Some(RateLimitInfo {
            remaining: 0,
            limit: self.info.as_ref().map(|i| i.limit).unwrap_or(5000),
            resets_at,
        });
    }

    /// Calculate backoff duration with exponential backoff.
    pub fn backoff_duration(&self) -> Duration {
        let base_secs = DEFAULT_BACKOFF_SECS;
        let multiplier = 2_u64.saturating_pow(self.consecutive_hits.min(5));
        Duration::from_secs(base_secs.saturating_mul(multiplier).min(900)) // Max 15 minutes
    }

    /// Check if we should wait before making a request.
    pub fn should_wait(&self) -> bool {
        if let Some(ref info) = self.info {
            if info.remaining == 0 && info.resets_at > Utc::now() {
                return true;
            }
        }
        false
    }

    /// Get the duration to wait before the rate limit resets.
    pub fn wait_duration(&self) -> Option<Duration> {
        self.info.as_ref().and_then(|info| {
            let now = Utc::now();
            if info.remaining == 0 && info.resets_at > now {
                let secs = (info.resets_at - now).num_seconds();
                if secs > 0 {
                    return Some(Duration::from_secs(secs as u64 + 1)); // Add 1 second buffer
                }
            }
            None
        })
    }
}

/// GitHub GraphQL API client.
pub struct GitHubClient {
    /// HTTP client
    http: reqwest::Client,

    /// Access token
    token: Arc<RwLock<Option<String>>>,

    /// API endpoint
    endpoint: String,

    /// Rate limit state
    rate_limit: Arc<RwLock<RateLimitState>>,
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self {
            http: reqwest::Client::new(),
            token: Arc::new(RwLock::new(None)),
            endpoint: "https://api.github.com/graphql".to_string(),
            rate_limit: Arc::new(RwLock::new(RateLimitState::default())),
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
            rate_limit: Arc::new(RwLock::new(RateLimitState::default())),
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

    /// Get the current rate limit info.
    pub async fn rate_limit_info(&self) -> Option<RateLimitInfo> {
        self.rate_limit.read().await.info.clone()
    }

    /// Parse rate limit headers from response.
    fn parse_rate_limit_headers(response: &reqwest::Response) -> Option<(u32, u32, i64)> {
        let headers = response.headers();
        
        let remaining = headers
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok())?;
        
        let limit = headers
            .get("x-ratelimit-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(5000);
        
        let reset = headers
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or_else(|| Utc::now().timestamp() + 3600);
        
        Some((remaining, limit, reset))
    }

    /// Wait for rate limit if needed.
    async fn wait_for_rate_limit(&self) -> Result<()> {
        let state = self.rate_limit.read().await;
        
        if let Some(wait_duration) = state.wait_duration() {
            drop(state); // Release read lock before sleeping
            
            let wait_secs = wait_duration.as_secs();
            tracing::warn!(
                "Rate limited by GitHub API. Waiting {} seconds before retry.",
                wait_secs
            );
            
            // Cap waiting time at 5 minutes for better UX
            let capped_duration = Duration::from_secs(wait_secs.min(300));
            sleep(capped_duration).await;
        }
        
        Ok(())
    }

    /// Execute a GraphQL query with rate limit handling and retry.
    pub async fn query<T>(&self, query: &str, variables: serde_json::Value) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut retries = 0;
        
        loop {
            // Check and wait for rate limit before making request
            self.wait_for_rate_limit().await?;
            
            match self.execute_query::<T>(query, variables.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // Check if it's a rate limit error
                    let is_rate_limit = matches!(
                        &e,
                        crate::error::AppError::GitHub(GitHubError::RateLimited { .. })
                    );
                    
                    if is_rate_limit && retries < MAX_RETRIES {
                        retries += 1;
                        
                        let state = self.rate_limit.read().await;
                        let backoff = state.backoff_duration();
                        drop(state);
                        
                        tracing::warn!(
                            "Rate limit hit. Retry {}/{} after {:?}",
                            retries,
                            MAX_RETRIES,
                            backoff
                        );
                        
                        sleep(backoff).await;
                        continue;
                    }
                    
                    return Err(e);
                }
            }
        }
    }

    /// Execute a single GraphQL query without retry logic.
    async fn execute_query<T>(&self, query: &str, variables: serde_json::Value) -> Result<T>
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

        // Update rate limit info from headers
        if let Some((remaining, limit, reset)) = Self::parse_rate_limit_headers(&response) {
            let mut state = self.rate_limit.write().await;
            state.update_from_headers(remaining, limit, reset);
        }

        // Handle rate limit response (403 or 429)
        if response.status() == reqwest::StatusCode::FORBIDDEN 
            || response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS 
        {
            let reset_timestamp = response
                .headers()
                .get("x-ratelimit-reset")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<i64>().ok());
            
            let mut state = self.rate_limit.write().await;
            state.mark_rate_limited(reset_timestamp);
            
            let reset_at = state.info
                .as_ref()
                .map(|i| i.resets_at.to_rfc3339())
                .unwrap_or_else(|| "unknown".to_string());
            
            return Err(GitHubError::RateLimited { reset_at }.into());
        }

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

    #[test]
    fn test_rate_limit_state_update_from_headers() {
        let mut state = RateLimitState::default();
        let reset_time = Utc::now().timestamp() + 3600;
        
        state.update_from_headers(100, 5000, reset_time);
        
        assert!(state.info.is_some());
        let info = state.info.unwrap();
        assert_eq!(info.remaining, 100);
        assert_eq!(info.limit, 5000);
        assert_eq!(state.consecutive_hits, 0);
    }

    #[test]
    fn test_rate_limit_state_mark_rate_limited() {
        let mut state = RateLimitState::default();
        
        state.mark_rate_limited(None);
        
        assert!(state.info.is_some());
        assert_eq!(state.consecutive_hits, 1);
        assert_eq!(state.info.as_ref().unwrap().remaining, 0);
        
        state.mark_rate_limited(None);
        assert_eq!(state.consecutive_hits, 2);
    }

    #[test]
    fn test_rate_limit_state_backoff_duration() {
        let mut state = RateLimitState::default();
        
        // First hit: 60 seconds
        state.mark_rate_limited(None);
        assert_eq!(state.backoff_duration().as_secs(), 120); // 60 * 2^1
        
        // Second hit: exponential backoff
        state.mark_rate_limited(None);
        assert_eq!(state.backoff_duration().as_secs(), 240); // 60 * 2^2
    }

    #[test]
    fn test_rate_limit_state_should_wait() {
        let mut state = RateLimitState::default();
        
        // No info yet - should not wait
        assert!(!state.should_wait());
        
        // Mark rate limited with future reset time
        let reset_time = Utc::now().timestamp() + 3600;
        state.mark_rate_limited(Some(reset_time));
        
        // Now should wait
        assert!(state.should_wait());
    }

    #[test]
    fn test_rate_limit_state_wait_duration() {
        let mut state = RateLimitState::default();
        
        // No info - no wait
        assert!(state.wait_duration().is_none());
        
        // Has remaining quota - no wait
        let reset_time = Utc::now().timestamp() + 3600;
        state.update_from_headers(100, 5000, reset_time);
        assert!(state.wait_duration().is_none());
        
        // Rate limited - should wait
        state.mark_rate_limited(Some(reset_time));
        let wait = state.wait_duration();
        assert!(wait.is_some());
        assert!(wait.unwrap().as_secs() > 0);
    }
}
