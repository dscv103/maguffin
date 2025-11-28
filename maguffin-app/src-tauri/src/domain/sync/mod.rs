//! Synchronization domain types.
//!
//! This module handles background data synchronization with GitHub.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Current sync state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SyncStatus {
    /// Sync is idle
    Idle {
        /// Last successful sync time
        last_sync: Option<DateTime<Utc>>,
    },

    /// Sync is in progress
    InProgress {
        /// When the sync started
        started_at: DateTime<Utc>,

        /// What is being synced
        current_task: Option<String>,
    },

    /// Sync failed
    Failed {
        /// Error message
        error: String,

        /// When the failure occurred
        failed_at: DateTime<Utc>,

        /// Number of consecutive failures
        failure_count: u32,
    },

    /// Rate limited by GitHub
    RateLimited {
        /// When the rate limit resets
        resets_at: DateTime<Utc>,
    },
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self::Idle { last_sync: None }
    }
}

/// A change detected during sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SyncChange {
    /// A PR was created
    PrCreated { number: i64, title: String },

    /// A PR was updated
    PrUpdated { number: i64, title: String },

    /// A PR was closed
    PrClosed { number: i64, merged: bool },

    /// A PR's review status changed
    PrReviewChanged { number: i64, new_status: String },

    /// A stack parent was merged
    StackParentMerged { stack_id: String, branch: String },
}

/// Rate limit information from GitHub.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Remaining requests in current window
    pub remaining: u32,

    /// Total requests allowed in window
    pub limit: u32,

    /// When the limit resets
    pub resets_at: DateTime<Utc>,
}

impl RateLimitInfo {
    /// Check if we're rate limited.
    pub fn is_limited(&self) -> bool {
        self.remaining == 0
    }

    /// Get percentage of rate limit used.
    pub fn usage_percent(&self) -> f64 {
        if self.limit == 0 {
            return 100.0;
        }
        ((self.limit - self.remaining) as f64 / self.limit as f64) * 100.0
    }

    /// Check if we should slow down (>80% used).
    pub fn should_slow_down(&self) -> bool {
        self.usage_percent() > 80.0
    }
}

/// Sync statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncStats {
    /// Total syncs performed
    pub total_syncs: u64,

    /// Successful syncs
    pub successful_syncs: u64,

    /// Failed syncs
    pub failed_syncs: u64,

    /// Total API requests made
    pub api_requests: u64,

    /// Average sync duration in milliseconds
    pub avg_sync_duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_status_default() {
        let status = SyncStatus::default();
        assert!(matches!(status, SyncStatus::Idle { .. }));
    }

    #[test]
    fn test_rate_limit_is_limited() {
        let info = RateLimitInfo {
            remaining: 0,
            limit: 5000,
            resets_at: Utc::now(),
        };
        assert!(info.is_limited());
    }

    #[test]
    fn test_rate_limit_should_slow_down() {
        let info = RateLimitInfo {
            remaining: 500,
            limit: 5000,
            resets_at: Utc::now(),
        };
        assert!(info.should_slow_down()); // 90% used
    }

    #[test]
    fn test_rate_limit_usage_percent() {
        let info = RateLimitInfo {
            remaining: 2500,
            limit: 5000,
            resets_at: Utc::now(),
        };
        assert!((info.usage_percent() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_sync_change_serialization() {
        let change = SyncChange::PrCreated {
            number: 42,
            title: "Test PR".to_string(),
        };
        let json = serde_json::to_string(&change).unwrap();
        assert!(json.contains("pr_created"));
    }
}
