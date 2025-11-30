//! Domain layer for the Maguffin application.
//!
//! This module contains the core business logic and domain types
//! for the Git client application.

pub mod auth;
pub mod pr;
pub mod repo;
pub mod stack;
pub mod sync;
pub mod template;

// Re-export commonly used types
pub use auth::{AuthState, AuthenticatedUser, DeviceFlowPending};
pub use pr::{
    Author, ChangeType, ChangedFile, Commit, CreatePrOptions, Label, MergeMethod, Mergeable,
    PrState, PullRequest, PullRequestDetails, Review, ReviewDecision, ReviewState,
};
pub use repo::{GitHubRemote, RecentRepository, Repository, SyncState};
pub use stack::{
    BranchStatus, ReconcileReport, RestackResult, RestackStatus, Stack, StackBranch, StackMetadata,
};
pub use sync::{RateLimitInfo, SyncChange, SyncStatus};
pub use template::{PrTemplate, TemplateContext};
