//! Stack Service.
//!
//! This module provides the service layer for managing stacked branches,
//! including creation, restacking, and reconciliation.

use crate::domain::stack::{
    BranchStatus, ReconcileReport, RestackConflict, RestackResult, RestackStatus, Stack,
    StackBranch, StackMetadata, Warning,
};
use crate::error::{GitError, Result};
use crate::git::{Git2Backend, GitOperations};
use crate::github::pr_service::PrService;
use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Service for managing stacked branches and PRs.
pub struct StackService {
    /// Path to the repository
    repo_path: PathBuf,

    /// Git backend for local operations
    git: Arc<RwLock<Git2Backend>>,

    /// PR service for GitHub operations
    pr_service: Option<Arc<PrService>>,

    /// Stack metadata
    metadata: Arc<RwLock<StackMetadata>>,
}

impl StackService {
    /// Create a new stack service.
    pub fn new(repo_path: PathBuf, git: Git2Backend) -> Result<Self> {
        let metadata_path = repo_path.join(".git").join("stack-metadata.json");
        let metadata = if metadata_path.exists() {
            let content = std::fs::read_to_string(&metadata_path)
                .map_err(|e| GitError::RepositoryNotFound(e.to_string()))?;
            serde_json::from_str(&content).unwrap_or_else(|_| StackMetadata::default())
        } else {
            StackMetadata::default()
        };

        Ok(Self {
            repo_path,
            git: Arc::new(RwLock::new(git)),
            pr_service: None,
            metadata: Arc::new(RwLock::new(metadata)),
        })
    }

    /// Set the PR service for GitHub operations.
    pub fn with_pr_service(mut self, pr_service: Arc<PrService>) -> Self {
        self.pr_service = Some(pr_service);
        self
    }

    /// Save metadata to disk.
    async fn save_metadata(&self) -> Result<()> {
        let metadata = self.metadata.read().await;
        let content = serde_json::to_string_pretty(&*metadata)
            .map_err(|e| GitError::RepositoryNotFound(e.to_string()))?;

        let metadata_path = self.repo_path.join(".git").join("stack-metadata.json");
        std::fs::write(&metadata_path, content)
            .map_err(|e| GitError::RepositoryNotFound(e.to_string()))?;

        Ok(())
    }

    /// Create a new stack rooted at the given branch.
    pub async fn create_stack(&self, root_branch: String) -> Result<Stack> {
        let stack = Stack::new(root_branch);

        let mut metadata = self.metadata.write().await;
        metadata.add_stack(stack.clone());
        drop(metadata);

        self.save_metadata().await?;

        Ok(stack)
    }

    /// Create a new branch on an existing stack.
    pub async fn create_stack_branch(
        &self,
        stack_id: uuid::Uuid,
        branch_name: String,
        parent_name: String,
    ) -> Result<StackBranch> {
        // Create the branch using git
        {
            let git = self.git.read().await;
            git.create_branch(&branch_name, &parent_name)?;
        }

        // Get the head SHA
        let head_sha = {
            let git = self.git.read().await;
            git.get_head_sha(&branch_name).ok()
        };

        // Add to metadata
        let mut branch = StackBranch::new(branch_name.clone(), parent_name);
        if let Some(sha) = head_sha {
            branch = branch.with_sha(sha);
        }

        let mut metadata = self.metadata.write().await;
        if let Some(stack) = metadata.stacks.iter_mut().find(|s| s.id == stack_id) {
            stack.add_branch(branch.clone());
        }
        drop(metadata);

        self.save_metadata().await?;

        Ok(branch)
    }

    /// Get all stacks.
    pub async fn list_stacks(&self) -> Vec<Stack> {
        let metadata = self.metadata.read().await;
        metadata.stacks.clone()
    }

    /// Get a specific stack by ID.
    pub async fn get_stack(&self, stack_id: uuid::Uuid) -> Option<Stack> {
        let metadata = self.metadata.read().await;
        metadata.find_stack(&stack_id).cloned()
    }

    /// Find the stack containing a branch.
    pub async fn find_stack_for_branch(&self, branch_name: &str) -> Option<Stack> {
        let metadata = self.metadata.read().await;
        metadata.find_stack_containing(branch_name).cloned()
    }

    /// Restack all branches after a parent has been merged.
    pub async fn restack(&self, stack_id: uuid::Uuid) -> Result<RestackResult> {
        let stack = self
            .get_stack(stack_id)
            .await
            .ok_or_else(|| GitError::Branch("Stack not found".to_string()))?;

        let mut result = RestackResult {
            status: RestackStatus::Success,
            restacked: Vec::new(),
            conflicts: Vec::new(),
            error: None,
        };

        // Get branches in topological order
        let branches = stack.topological_order();

        for branch in branches {
            // Check if branch needs restacking
            let needs_restack = {
                let git = self.git.read().await;
                git.needs_rebase(&branch.name, &branch.parent)
                    .unwrap_or(true)
            };

            if !needs_restack {
                result.restacked.push(branch.name.clone());
                continue;
            }

            // Perform the rebase
            let rebase_result = {
                let git = self.git.read().await;
                git.rebase(&branch.name, &branch.parent)
            };

            match rebase_result {
                Ok(_) => {
                    result.restacked.push(branch.name.clone());

                    // Update branch status in metadata
                    let mut metadata = self.metadata.write().await;
                    if let Some(s) = metadata.stacks.iter_mut().find(|s| s.id == stack_id) {
                        if let Some(b) = s.find_branch_mut(&branch.name) {
                            b.status = BranchStatus::UpToDate;
                            // Update head SHA
                            if let Ok(sha) = self.git.read().await.get_head_sha(&branch.name) {
                                b.head_sha = Some(sha);
                            }
                        }
                    }
                }
                Err(e) => {
                    // Check if it's a conflict
                    if e.to_string().contains("conflict") {
                        let conflict_files = self.get_conflict_files().await;
                        result.conflicts.push(RestackConflict {
                            branch: branch.name.clone(),
                            files: conflict_files,
                        });
                        result.status = RestackStatus::Conflicts;

                        // Abort the rebase
                        let _ = self.git.read().await.abort_rebase();

                        // Update branch status
                        let mut metadata = self.metadata.write().await;
                        if let Some(s) = metadata.stacks.iter_mut().find(|s| s.id == stack_id) {
                            if let Some(b) = s.find_branch_mut(&branch.name) {
                                b.status = BranchStatus::Conflicted;
                            }
                        }

                        break;
                    } else {
                        result.status = RestackStatus::Failed;
                        result.error = Some(e.to_string());
                        break;
                    }
                }
            }
        }

        // Update last sync time
        {
            let mut metadata = self.metadata.write().await;
            metadata.last_sync = Some(Utc::now());
        }

        self.save_metadata().await?;

        Ok(result)
    }

    /// Get files with conflicts.
    async fn get_conflict_files(&self) -> Vec<String> {
        // In a real implementation, this would check the git index for conflicts
        Vec::new()
    }

    /// Reconcile stack metadata with actual Git state.
    pub async fn reconcile(&self) -> Result<ReconcileReport> {
        let mut report = ReconcileReport::new();

        let mut metadata = self.metadata.write().await;
        let git = self.git.read().await;

        for stack in &mut metadata.stacks {
            for branch in &mut stack.branches {
                // Check if branch exists
                match git.branch_exists(&branch.name) {
                    Ok(true) => {
                        // Check if parent is still ancestor
                        match git.is_ancestor(&branch.parent, &branch.name) {
                            Ok(true) => {
                                // Check if branch needs rebase
                                if git
                                    .needs_rebase(&branch.name, &branch.parent)
                                    .unwrap_or(false)
                                {
                                    branch.status = BranchStatus::NeedsRebase;
                                } else {
                                    branch.status = BranchStatus::UpToDate;
                                }
                            }
                            Ok(false) => {
                                report.add_warning(branch.name.clone(), Warning::ParentNotAncestor);
                                branch.status = BranchStatus::NeedsRebase;
                            }
                            Err(_) => {
                                // Parent might not exist
                                if !git.branch_exists(&branch.parent).unwrap_or(false) {
                                    report.add_warning(branch.name.clone(), Warning::ParentDeleted);
                                }
                            }
                        }

                        // Update head SHA
                        if let Ok(sha) = git.get_head_sha(&branch.name) {
                            if branch.head_sha.as_ref() != Some(&sha) {
                                if branch.head_sha.is_some() {
                                    report.add_warning(
                                        branch.name.clone(),
                                        Warning::ExternallyModified,
                                    );
                                }
                                branch.head_sha = Some(sha);
                            }
                        }
                    }
                    Ok(false) => {
                        branch.status = BranchStatus::Orphaned;
                        report.add_orphan(branch.name.clone());
                    }
                    Err(_) => {
                        branch.status = BranchStatus::Unknown;
                    }
                }
            }
        }

        drop(git);
        drop(metadata);

        self.save_metadata().await?;

        Ok(report)
    }

    /// Update PR base branch after parent is merged.
    pub async fn update_pr_base(&self, branch_name: &str, new_base: &str) -> Result<()> {
        let _pr_service = self
            .pr_service
            .as_ref()
            .ok_or_else(|| GitError::Branch("PR service not configured".to_string()))?;

        // Find the branch and its PR number
        let metadata = self.metadata.read().await;
        let pr_number = metadata
            .stacks
            .iter()
            .flat_map(|s| s.branches.iter())
            .find(|b| b.name == branch_name)
            .and_then(|b| b.pr_number);

        if let Some(_pr_number) = pr_number {
            // In a full implementation, we would update the PR base using the GitHub API
            // This requires the updatePullRequest mutation
            tracing::info!("Would update PR for {} to target {}", branch_name, new_base);
        }

        Ok(())
    }

    /// Associate a PR number with a branch.
    pub async fn set_branch_pr(&self, branch_name: &str, pr_number: i64) -> Result<()> {
        let mut metadata = self.metadata.write().await;

        for stack in &mut metadata.stacks {
            if let Some(branch) = stack.find_branch_mut(branch_name) {
                branch.pr_number = Some(pr_number);
                break;
            }
        }

        drop(metadata);
        self.save_metadata().await?;

        Ok(())
    }

    /// Remove a branch from its stack.
    pub async fn remove_branch(&self, branch_name: &str) -> Result<()> {
        let mut metadata = self.metadata.write().await;

        for stack in &mut metadata.stacks {
            stack.branches.retain(|b| b.name != branch_name);
        }

        drop(metadata);
        self.save_metadata().await?;

        Ok(())
    }

    /// Delete a stack.
    pub async fn delete_stack(&self, stack_id: uuid::Uuid) -> Result<()> {
        let mut metadata = self.metadata.write().await;
        metadata.stacks.retain(|s| s.id != stack_id);
        drop(metadata);

        self.save_metadata().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_metadata_default() {
        let metadata = StackMetadata::default();
        assert!(metadata.stacks.is_empty());
        assert_eq!(metadata.version, 1);
    }
}
