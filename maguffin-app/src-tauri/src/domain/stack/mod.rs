//! Stack domain types for managing stacked branches and PRs.
//!
//! This module implements the Graphite-style stacked PR workflow,
//! allowing branches to form parent-child relationships.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A stack of related branches with parent-child relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack {
    /// Unique identifier for this stack
    pub id: Uuid,

    /// Root branch (usually main or develop)
    pub root: String,

    /// Branches in this stack
    pub branches: Vec<StackBranch>,

    /// When the stack was created
    pub created_at: DateTime<Utc>,

    /// When the stack was last modified
    pub updated_at: DateTime<Utc>,
}

impl Stack {
    /// Create a new stack rooted at the given branch.
    pub fn new(root: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            root,
            branches: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a branch to the stack.
    pub fn add_branch(&mut self, branch: StackBranch) {
        self.branches.push(branch);
        self.updated_at = Utc::now();
    }

    /// Find a branch by name.
    pub fn find_branch(&self, name: &str) -> Option<&StackBranch> {
        self.branches.iter().find(|b| b.name == name)
    }

    /// Find a branch by name (mutable).
    pub fn find_branch_mut(&mut self, name: &str) -> Option<&mut StackBranch> {
        self.branches.iter_mut().find(|b| b.name == name)
    }

    /// Get all children of a branch.
    pub fn children_of(&self, parent: &str) -> Vec<&StackBranch> {
        self.branches
            .iter()
            .filter(|b| b.parent == parent)
            .collect()
    }

    /// Get branches in topological order (parents before children).
    pub fn topological_order(&self) -> Vec<&StackBranch> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![&self.root];

        while let Some(current) = stack.pop() {
            if visited.contains(current) {
                continue;
            }
            visited.insert(current.to_string());

            // Add all children to process
            for branch in self.children_of(current) {
                if !visited.contains(&branch.name) {
                    result.push(branch);
                    stack.push(&branch.name);
                }
            }
        }

        result
    }
}

/// A branch in a stack with its parent relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackBranch {
    /// Branch name
    pub name: String,

    /// Parent branch name (can be the root branch)
    pub parent: String,

    /// Associated PR number (if any)
    pub pr_number: Option<i64>,

    /// Current sync status
    pub status: BranchStatus,

    /// When this branch was added to the stack
    pub created_at: DateTime<Utc>,

    /// Last known commit SHA
    pub head_sha: Option<String>,
}

impl StackBranch {
    /// Create a new stack branch.
    pub fn new(name: String, parent: String) -> Self {
        Self {
            name,
            parent,
            pr_number: None,
            status: BranchStatus::UpToDate,
            created_at: Utc::now(),
            head_sha: None,
        }
    }

    /// Set the associated PR number.
    pub fn with_pr(mut self, pr_number: i64) -> Self {
        self.pr_number = Some(pr_number);
        self
    }

    /// Set the head SHA.
    pub fn with_sha(mut self, sha: String) -> Self {
        self.head_sha = Some(sha);
        self
    }
}

/// Status of a branch relative to its parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BranchStatus {
    /// Branch is up to date with parent
    UpToDate,

    /// Parent has new commits, branch needs rebase
    NeedsRebase,

    /// Branch has merge conflicts with parent
    Conflicted,

    /// Branch reference doesn't exist (deleted externally)
    Orphaned,

    /// Status is unknown (needs refresh)
    Unknown,
}

/// Metadata file structure for persisting stack information.
/// Stored in `.git/stack-metadata.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackMetadata {
    /// Schema version
    pub version: u32,

    /// All stacks in this repository
    pub stacks: Vec<Stack>,

    /// Last sync timestamp
    pub last_sync: Option<DateTime<Utc>>,
}

impl Default for StackMetadata {
    fn default() -> Self {
        Self {
            version: 1,
            stacks: Vec::new(),
            last_sync: None,
        }
    }
}

impl StackMetadata {
    /// Find a stack by ID.
    pub fn find_stack(&self, id: &Uuid) -> Option<&Stack> {
        self.stacks.iter().find(|s| s.id == *id)
    }

    /// Find a stack containing a specific branch.
    pub fn find_stack_containing(&self, branch: &str) -> Option<&Stack> {
        self.stacks
            .iter()
            .find(|s| s.branches.iter().any(|b| b.name == branch))
    }

    /// Add a new stack.
    pub fn add_stack(&mut self, stack: Stack) {
        self.stacks.push(stack);
    }
}

/// Result of a restack operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestackResult {
    /// Status of the restack
    pub status: RestackStatus,

    /// Branches that were successfully restacked
    pub restacked: Vec<String>,

    /// Branches with conflicts (if any)
    pub conflicts: Vec<RestackConflict>,

    /// Error message (if failed)
    pub error: Option<String>,
}

/// Status of a restack operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RestackStatus {
    /// All branches restacked successfully
    Success,

    /// Restack paused due to conflicts
    Conflicts,

    /// Restack failed with an error
    Failed,
}

/// Information about a conflict during restacking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestackConflict {
    /// Branch with the conflict
    pub branch: String,

    /// Files with conflicts
    pub files: Vec<String>,
}

/// Report from reconciling stack metadata with actual Git state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReconcileReport {
    /// Branches that are orphaned (no longer exist in Git)
    pub orphaned: Vec<String>,

    /// Warnings about branch state
    pub warnings: Vec<ReconcileWarning>,
}

impl ReconcileReport {
    /// Create a new empty report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an orphaned branch.
    pub fn add_orphan(&mut self, branch: String) {
        self.orphaned.push(branch);
    }

    /// Add a warning.
    pub fn add_warning(&mut self, branch: String, warning: Warning) {
        self.warnings.push(ReconcileWarning { branch, warning });
    }
}

/// A warning about a branch's state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconcileWarning {
    /// Branch name
    pub branch: String,

    /// Warning type
    pub warning: Warning,
}

/// Types of reconciliation warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Warning {
    /// Parent branch is not an ancestor of this branch
    ParentNotAncestor,

    /// Branch has been modified externally
    ExternallyModified,

    /// Parent branch was deleted
    ParentDeleted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_creation() {
        let stack = Stack::new("main".to_string());
        assert_eq!(stack.root, "main");
        assert!(stack.branches.is_empty());
    }

    #[test]
    fn test_add_branch_to_stack() {
        let mut stack = Stack::new("main".to_string());
        let branch = StackBranch::new("feature-a".to_string(), "main".to_string());
        stack.add_branch(branch);

        assert_eq!(stack.branches.len(), 1);
        assert!(stack.find_branch("feature-a").is_some());
    }

    #[test]
    fn test_children_of() {
        let mut stack = Stack::new("main".to_string());
        stack.add_branch(StackBranch::new(
            "feature-a".to_string(),
            "main".to_string(),
        ));
        stack.add_branch(StackBranch::new(
            "feature-b".to_string(),
            "feature-a".to_string(),
        ));
        stack.add_branch(StackBranch::new(
            "feature-c".to_string(),
            "main".to_string(),
        ));

        let children = stack.children_of("main");
        assert_eq!(children.len(), 2);

        let children_a = stack.children_of("feature-a");
        assert_eq!(children_a.len(), 1);
        assert_eq!(children_a[0].name, "feature-b");
    }

    #[test]
    fn test_branch_status_serialization() {
        let status = BranchStatus::NeedsRebase;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"needs_rebase\"");
    }

    #[test]
    fn test_stack_metadata_find_stack_containing() {
        let mut metadata = StackMetadata::default();
        let mut stack = Stack::new("main".to_string());
        stack.add_branch(StackBranch::new(
            "feature-a".to_string(),
            "main".to_string(),
        ));
        metadata.add_stack(stack);

        assert!(metadata.find_stack_containing("feature-a").is_some());
        assert!(metadata.find_stack_containing("nonexistent").is_none());
    }
}
