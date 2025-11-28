//! Background Sync Service.
//!
//! This module provides the background synchronization service that keeps
//! local data in sync with GitHub. It polls for changes at configurable
//! intervals and handles rate limiting gracefully.

use crate::config::SyncConfig;
use crate::domain::sync::{RateLimitInfo, SyncChange, SyncStats, SyncStatus};
use crate::domain::PullRequest;
use crate::error::Result;
use crate::github::{GitHubClient, PrService};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::{interval, Duration};

/// Message types for controlling the sync service.
#[derive(Debug, Clone)]
pub enum SyncCommand {
    /// Start background sync
    Start,
    /// Stop background sync
    Stop,
    /// Trigger immediate sync
    SyncNow,
    /// Update sync configuration
    UpdateConfig(SyncConfig),
}

/// Event types emitted by the sync service.
#[derive(Debug, Clone)]
pub enum SyncEvent {
    /// Sync status changed
    StatusChanged(SyncStatus),
    /// Changes detected during sync
    ChangesDetected(Vec<SyncChange>),
    /// Rate limit info updated
    RateLimitUpdated(RateLimitInfo),
    /// Error occurred
    Error(String),
}

/// Background sync service for keeping data in sync with GitHub.
pub struct SyncService {
    /// GitHub client for API calls
    github_client: Arc<GitHubClient>,

    /// Current sync status
    status: Arc<RwLock<SyncStatus>>,

    /// Sync configuration
    config: Arc<RwLock<SyncConfig>>,

    /// Rate limit info
    rate_limit: Arc<RwLock<Option<RateLimitInfo>>>,

    /// Sync statistics
    stats: Arc<RwLock<SyncStats>>,

    /// Cached PR data for change detection
    cached_prs: Arc<RwLock<Vec<PullRequest>>>,

    /// Repository context (owner, repo)
    repo_context: Arc<RwLock<Option<(String, String)>>>,

    /// Event broadcast sender
    event_tx: broadcast::Sender<SyncEvent>,

    /// Command receiver - stored for task management
    command_rx: Arc<RwLock<Option<mpsc::Receiver<SyncCommand>>>>,

    /// Command sender for external control
    command_tx: mpsc::Sender<SyncCommand>,

    /// Whether the service is running
    running: Arc<RwLock<bool>>,
}

impl SyncService {
    /// Create a new sync service.
    pub fn new(github_client: Arc<GitHubClient>, config: SyncConfig) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        let (command_tx, command_rx) = mpsc::channel(10);

        Self {
            github_client,
            status: Arc::new(RwLock::new(SyncStatus::default())),
            config: Arc::new(RwLock::new(config)),
            rate_limit: Arc::new(RwLock::new(None)),
            stats: Arc::new(RwLock::new(SyncStats::default())),
            cached_prs: Arc::new(RwLock::new(Vec::new())),
            repo_context: Arc::new(RwLock::new(None)),
            event_tx,
            command_rx: Arc::new(RwLock::new(Some(command_rx))),
            command_tx,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Set the repository context.
    pub async fn set_repository(&self, owner: String, repo: String) {
        *self.repo_context.write().await = Some((owner, repo));
        // Clear cached data when switching repos
        self.cached_prs.write().await.clear();
    }

    /// Clear the repository context.
    pub async fn clear_repository(&self) {
        *self.repo_context.write().await = None;
        self.cached_prs.write().await.clear();
    }

    /// Get a receiver for sync events.
    pub fn subscribe(&self) -> broadcast::Receiver<SyncEvent> {
        self.event_tx.subscribe()
    }

    /// Get a command sender for controlling the service.
    pub fn command_sender(&self) -> mpsc::Sender<SyncCommand> {
        self.command_tx.clone()
    }

    /// Get the current sync status.
    pub async fn status(&self) -> SyncStatus {
        self.status.read().await.clone()
    }

    /// Get the current rate limit info.
    pub async fn rate_limit(&self) -> Option<RateLimitInfo> {
        self.rate_limit.read().await.clone()
    }

    /// Get sync statistics.
    pub async fn stats(&self) -> SyncStats {
        self.stats.read().await.clone()
    }

    /// Check if sync is enabled and should run.
    pub async fn should_sync(&self) -> bool {
        let config = self.config.read().await;
        if !config.enabled {
            return false;
        }

        // Check rate limit
        if let Some(ref rate_limit) = *self.rate_limit.read().await {
            if rate_limit.is_limited() {
                return false;
            }
        }

        // Check if we have a repo context
        self.repo_context.read().await.is_some()
    }

    /// Start the background sync loop.
    pub async fn start(&self) {
        // Take ownership of the receiver
        let command_rx = {
            let mut rx_guard = self.command_rx.write().await;
            rx_guard.take()
        };

        let Some(mut command_rx) = command_rx else {
            tracing::warn!("SyncService already started");
            return;
        };

        *self.running.write().await = true;
        tracing::info!("Starting background sync service");

        let config = self.config.clone();
        let status = self.status.clone();
        let rate_limit = self.rate_limit.clone();
        let stats = self.stats.clone();
        let cached_prs = self.cached_prs.clone();
        let repo_context = self.repo_context.clone();
        let github_client = self.github_client.clone();
        let event_tx = self.event_tx.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut sync_interval = interval(Duration::from_secs(60));

            loop {
                // Update interval from config
                let interval_secs = config.read().await.interval_secs;
                sync_interval = interval(Duration::from_secs(interval_secs));

                tokio::select! {
                    // Timer tick
                    _ = sync_interval.tick() => {
                        if *running.read().await {
                            let should_sync = {
                                let cfg = config.read().await;
                                cfg.enabled
                            };

                            if should_sync {
                                Self::perform_sync(
                                    &github_client,
                                    &repo_context,
                                    &cached_prs,
                                    &status,
                                    &stats,
                                    &rate_limit,
                                    &event_tx,
                                ).await;
                            }
                        }
                    }

                    // Command received
                    Some(cmd) = command_rx.recv() => {
                        match cmd {
                            SyncCommand::Start => {
                                *running.write().await = true;
                                tracing::info!("Sync service started");
                            }
                            SyncCommand::Stop => {
                                *running.write().await = false;
                                tracing::info!("Sync service stopped");
                            }
                            SyncCommand::SyncNow => {
                                tracing::info!("Manual sync triggered");
                                Self::perform_sync(
                                    &github_client,
                                    &repo_context,
                                    &cached_prs,
                                    &status,
                                    &stats,
                                    &rate_limit,
                                    &event_tx,
                                ).await;
                            }
                            SyncCommand::UpdateConfig(new_config) => {
                                *config.write().await = new_config;
                                tracing::info!("Sync config updated");
                            }
                        }
                    }
                }
            }
        });
    }

    /// Perform a single sync operation.
    async fn perform_sync(
        github_client: &Arc<GitHubClient>,
        repo_context: &Arc<RwLock<Option<(String, String)>>>,
        cached_prs: &Arc<RwLock<Vec<PullRequest>>>,
        status: &Arc<RwLock<SyncStatus>>,
        stats: &Arc<RwLock<SyncStats>>,
        rate_limit: &Arc<RwLock<Option<RateLimitInfo>>>,
        event_tx: &broadcast::Sender<SyncEvent>,
    ) {
        // Get repo context
        let context = repo_context.read().await.clone();
        let Some((owner, repo)) = context else {
            return;
        };

        // Update status to in progress
        let started_at = Utc::now();
        *status.write().await = SyncStatus::InProgress {
            started_at,
            current_task: Some("Fetching pull requests".to_string()),
        };
        let _ = event_tx.send(SyncEvent::StatusChanged(SyncStatus::InProgress {
            started_at,
            current_task: Some("Fetching pull requests".to_string()),
        }));

        // Create PR service
        let pr_service = PrService::new(github_client.clone(), owner, repo);

        // Fetch PRs
        match pr_service.list_prs(None).await {
            Ok(new_prs) => {
                // Detect changes
                let changes = {
                    let old_prs = cached_prs.read().await;
                    Self::detect_changes(&old_prs, &new_prs)
                };

                // Update cache
                *cached_prs.write().await = new_prs;

                // Emit changes if any
                if !changes.is_empty() {
                    let _ = event_tx.send(SyncEvent::ChangesDetected(changes));
                }

                // Update status
                let now = Utc::now();
                *status.write().await = SyncStatus::Idle {
                    last_sync: Some(now),
                };
                let _ = event_tx.send(SyncEvent::StatusChanged(SyncStatus::Idle {
                    last_sync: Some(now),
                }));

                // Update stats
                {
                    let mut s = stats.write().await;
                    s.total_syncs += 1;
                    s.successful_syncs += 1;
                    s.api_requests += 1;
                    let duration = (Utc::now() - started_at).num_milliseconds() as u64;
                    s.avg_sync_duration_ms =
                        (s.avg_sync_duration_ms * (s.successful_syncs - 1) + duration)
                            / s.successful_syncs;
                }
            }
            Err(e) => {
                let error_msg = e.to_string();
                tracing::error!("Sync failed: {}", error_msg);

                // Check if it's a rate limit error
                if error_msg.contains("rate limit") {
                    let resets_at = Utc::now() + chrono::Duration::minutes(15);
                    *status.write().await = SyncStatus::RateLimited { resets_at };
                    *rate_limit.write().await = Some(RateLimitInfo {
                        remaining: 0,
                        limit: 5000,
                        resets_at,
                    });
                    let _ = event_tx.send(SyncEvent::RateLimitUpdated(RateLimitInfo {
                        remaining: 0,
                        limit: 5000,
                        resets_at,
                    }));
                } else {
                    // Update failure status
                    let failed_at = Utc::now();
                    let failure_count = {
                        let current = status.read().await;
                        if let SyncStatus::Failed { failure_count, .. } = &*current {
                            failure_count + 1
                        } else {
                            1
                        }
                    };

                    *status.write().await = SyncStatus::Failed {
                        error: error_msg.clone(),
                        failed_at,
                        failure_count,
                    };
                    let _ = event_tx.send(SyncEvent::StatusChanged(SyncStatus::Failed {
                        error: error_msg.clone(),
                        failed_at,
                        failure_count,
                    }));
                }

                // Update stats
                {
                    let mut s = stats.write().await;
                    s.total_syncs += 1;
                    s.failed_syncs += 1;
                    s.api_requests += 1;
                }

                let _ = event_tx.send(SyncEvent::Error(error_msg));
            }
        }
    }

    /// Detect changes between old and new PR lists.
    fn detect_changes(old_prs: &[PullRequest], new_prs: &[PullRequest]) -> Vec<SyncChange> {
        let mut changes = Vec::new();

        // Build maps for efficient lookup
        let old_map: std::collections::HashMap<i64, &PullRequest> =
            old_prs.iter().map(|pr| (pr.number, pr)).collect();
        let new_map: std::collections::HashMap<i64, &PullRequest> =
            new_prs.iter().map(|pr| (pr.number, pr)).collect();

        // Check for new and updated PRs
        for new_pr in new_prs {
            match old_map.get(&new_pr.number) {
                Some(old_pr) => {
                    // Check for updates
                    if old_pr.updated_at != new_pr.updated_at {
                        changes.push(SyncChange::PrUpdated {
                            number: new_pr.number,
                            title: new_pr.title.clone(),
                        });
                    }
                    // Check for review status change
                    if old_pr.review_decision != new_pr.review_decision {
                        let new_status = new_pr
                            .review_decision
                            .as_ref()
                            .map(|d| format!("{:?}", d))
                            .unwrap_or_else(|| "pending".to_string());
                        changes.push(SyncChange::PrReviewChanged {
                            number: new_pr.number,
                            new_status,
                        });
                    }
                }
                None => {
                    // New PR
                    changes.push(SyncChange::PrCreated {
                        number: new_pr.number,
                        title: new_pr.title.clone(),
                    });
                }
            }
        }

        // Check for closed PRs
        for old_pr in old_prs {
            if !new_map.contains_key(&old_pr.number) {
                changes.push(SyncChange::PrClosed {
                    number: old_pr.number,
                    merged: old_pr.state == crate::domain::pr::PrState::Merged,
                });
            }
        }

        changes
    }

    /// Trigger an immediate sync.
    pub async fn sync_now(&self) -> Result<()> {
        self.command_tx
            .send(SyncCommand::SyncNow)
            .await
            .map_err(|e| crate::error::AppError::from(e.to_string()))
    }

    /// Update the sync configuration.
    pub async fn update_config(&self, config: SyncConfig) -> Result<()> {
        self.command_tx
            .send(SyncCommand::UpdateConfig(config))
            .await
            .map_err(|e| crate::error::AppError::from(e.to_string()))
    }

    /// Stop the sync service.
    pub async fn stop(&self) -> Result<()> {
        *self.running.write().await = false;
        self.command_tx
            .send(SyncCommand::Stop)
            .await
            .map_err(|e| crate::error::AppError::from(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::pr::{Author, Mergeable, PrState};

    fn make_pr(number: i64, title: &str, updated_at: DateTime<Utc>) -> PullRequest {
        PullRequest {
            id: format!("PR_{}", number),
            number,
            title: title.to_string(),
            body: None,
            state: PrState::Open,
            is_draft: false,
            author: Author {
                login: "test".to_string(),
                avatar_url: String::new(),
            },
            head_ref: "feature".to_string(),
            base_ref: "main".to_string(),
            labels: Vec::new(),
            review_decision: None,
            mergeable: Mergeable::Unknown,
            created_at: Utc::now(),
            updated_at,
            commit_count: 1,
            additions: 0,
            deletions: 0,
            changed_files: 0,
        }
    }

    #[test]
    fn test_detect_new_pr() {
        let old_prs = vec![];
        let new_prs = vec![make_pr(1, "New PR", Utc::now())];

        let changes = SyncService::detect_changes(&old_prs, &new_prs);
        assert_eq!(changes.len(), 1);
        assert!(matches!(changes[0], SyncChange::PrCreated { number: 1, .. }));
    }

    #[test]
    fn test_detect_closed_pr() {
        let old_prs = vec![make_pr(1, "Old PR", Utc::now())];
        let new_prs = vec![];

        let changes = SyncService::detect_changes(&old_prs, &new_prs);
        assert_eq!(changes.len(), 1);
        assert!(matches!(changes[0], SyncChange::PrClosed { number: 1, .. }));
    }

    #[test]
    fn test_detect_updated_pr() {
        let now = Utc::now();
        let old_prs = vec![make_pr(1, "PR", now - chrono::Duration::hours(1))];
        let new_prs = vec![make_pr(1, "PR", now)];

        let changes = SyncService::detect_changes(&old_prs, &new_prs);
        assert_eq!(changes.len(), 1);
        assert!(matches!(changes[0], SyncChange::PrUpdated { number: 1, .. }));
    }

    #[test]
    fn test_no_changes() {
        let now = Utc::now();
        let prs = vec![make_pr(1, "PR", now)];

        let changes = SyncService::detect_changes(&prs, &prs);
        assert!(changes.is_empty());
    }
}
