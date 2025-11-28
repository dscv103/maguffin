//! Configuration management for the Maguffin application.
//!
//! Handles user preferences, sync intervals, and other configurable settings.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Application configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    /// Sync settings
    pub sync: SyncConfig,

    /// UI preferences
    pub ui: UiConfig,

    /// GitHub settings
    pub github: GitHubConfig,
}

/// Configuration for background synchronization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Interval between sync operations in seconds (default: 60)
    pub interval_secs: u64,

    /// Whether background sync is enabled
    pub enabled: bool,

    /// Whether to sync on app startup
    pub sync_on_startup: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            interval_secs: 60,
            enabled: true,
            sync_on_startup: true,
        }
    }
}

impl SyncConfig {
    /// Get the sync interval as a Duration.
    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.interval_secs)
    }
}

/// UI preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Theme preference
    pub theme: Theme,

    /// Default PR list sort order
    pub default_sort: PrSortOrder,

    /// Number of PRs to show per page
    pub page_size: usize,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            default_sort: PrSortOrder::UpdatedAt,
            page_size: 25,
        }
    }
}

/// Theme preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    System,
}

/// PR list sort order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrSortOrder {
    CreatedAt,
    UpdatedAt,
    ReviewStatus,
}

/// GitHub-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// GitHub API endpoint (for enterprise support in the future)
    pub api_endpoint: String,

    /// Maximum concurrent API requests
    pub max_concurrent_requests: usize,

    /// OAuth client ID for device flow authentication
    pub oauth_client_id: String,
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            api_endpoint: "https://api.github.com/graphql".to_string(),
            max_concurrent_requests: 5,
            // Default OAuth client ID for Maguffin app
            // Can be overridden via environment variable MAGUFFIN_GITHUB_CLIENT_ID
            oauth_client_id: std::env::var("MAGUFFIN_GITHUB_CLIENT_ID")
                .unwrap_or_else(|_| "Ov23liYwNsRRRrKOQCvj".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.sync.interval_secs, 60);
        assert!(config.sync.enabled);
        assert_eq!(config.ui.theme, Theme::System);
    }

    #[test]
    fn test_sync_interval_duration() {
        let config = SyncConfig {
            interval_secs: 120,
            ..Default::default()
        };
        assert_eq!(config.interval(), Duration::from_secs(120));
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.sync.interval_secs, config.sync.interval_secs);
    }
}
