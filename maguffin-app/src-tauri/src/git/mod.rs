//! Git operations module.
//!
//! This module provides abstractions for Git operations using git2 (libgit2)
//! with a CLI fallback for complex operations.

use crate::error::{GitError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// State of an in-progress rebase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebaseState {
    /// The branch being rebased
    pub branch: Option<String>,
    /// The target commit/branch we're rebasing onto
    pub onto: Option<String>,
}

/// Trait for Git operations, allowing different implementations.
pub trait GitOperations: Send {
    /// Check out a branch.
    fn checkout_branch(&self, branch: &str) -> Result<()>;

    /// Create a new branch from a base.
    fn create_branch(&self, name: &str, from: &str) -> Result<()>;

    /// Get the current branch name.
    fn current_branch(&self) -> Result<String>;

    /// Fetch from a remote.
    fn fetch(&self, remote: &str) -> Result<()>;

    /// Check if a branch exists.
    fn branch_exists(&self, name: &str) -> Result<bool>;

    /// Get the default branch name (main or master).
    fn default_branch(&self) -> Result<String>;
}

/// Git2-based implementation of Git operations.
pub struct Git2Backend {
    repo: git2::Repository,
}

impl Git2Backend {
    /// Open a repository at the given path.
    pub fn open(path: &Path) -> Result<Self> {
        let repo = git2::Repository::open(path)
            .map_err(|e| GitError::RepositoryNotFound(e.to_string()))?;
        Ok(Self { repo })
    }

    /// Discover a repository starting from the given path.
    pub fn discover(path: &Path) -> Result<Self> {
        let repo = git2::Repository::discover(path)
            .map_err(|e| GitError::RepositoryNotFound(e.to_string()))?;
        Ok(Self { repo })
    }

    /// Get the repository path.
    pub fn path(&self) -> &Path {
        self.repo.path()
    }

    /// Get the working directory path.
    pub fn workdir(&self) -> Option<&Path> {
        self.repo.workdir()
    }

    /// Get the HEAD SHA of a branch.
    pub fn get_head_sha(&self, branch: &str) -> Result<String> {
        let branch = self
            .repo
            .find_branch(branch, git2::BranchType::Local)
            .map_err(|e| GitError::Branch(format!("Branch not found: {}", e)))?;

        let commit = branch
            .get()
            .peel_to_commit()
            .map_err(|e| GitError::Branch(format!("Failed to get commit: {}", e)))?;

        Ok(commit.id().to_string())
    }

    /// Check if one branch is an ancestor of another.
    pub fn is_ancestor(&self, ancestor: &str, descendant: &str) -> Result<bool> {
        let ancestor_oid = self
            .repo
            .revparse_single(&format!("refs/heads/{}", ancestor))
            .map_err(|e| GitError::Branch(format!("Ancestor branch not found: {}", e)))?
            .id();

        let descendant_oid = self
            .repo
            .revparse_single(&format!("refs/heads/{}", descendant))
            .map_err(|e| GitError::Branch(format!("Descendant branch not found: {}", e)))?
            .id();

        let result = self
            .repo
            .graph_descendant_of(descendant_oid, ancestor_oid)
            .map_err(|e| GitError::Branch(format!("Failed to check ancestry: {}", e)))?;

        Ok(result)
    }

    /// Check if a branch needs rebasing onto its parent.
    pub fn needs_rebase(&self, branch: &str, parent: &str) -> Result<bool> {
        let parent_oid = self
            .repo
            .revparse_single(&format!("refs/heads/{}", parent))
            .map_err(|e| GitError::Branch(format!("Parent branch not found: {}", e)))?
            .id();

        let branch_ref = self
            .repo
            .find_branch(branch, git2::BranchType::Local)
            .map_err(|e| GitError::Branch(format!("Branch not found: {}", e)))?;

        let branch_commit = branch_ref
            .get()
            .peel_to_commit()
            .map_err(|e| GitError::Branch(format!("Failed to get commit: {}", e)))?;

        // Find the merge base
        let merge_base = self
            .repo
            .merge_base(branch_commit.id(), parent_oid)
            .map_err(|e| GitError::Branch(format!("Failed to find merge base: {}", e)))?;

        // If the merge base is the same as the parent head, no rebase needed
        Ok(merge_base != parent_oid)
    }

    /// Perform a rebase of a branch onto another branch.
    /// Note: This is a simplified implementation. Complex rebases may require CLI fallback.
    pub fn rebase(&self, branch: &str, onto: &str) -> Result<()> {
        // For complex rebases, we shell out to git CLI
        let workdir = self
            .workdir()
            .ok_or_else(|| GitError::RepositoryNotFound("No working directory".to_string()))?;

        let output = std::process::Command::new("git")
            .args(["rebase", onto, branch])
            .current_dir(workdir)
            .output()
            .map_err(|e| GitError::RebaseFailed(e.to_string()))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("conflict") || stderr.contains("CONFLICT") {
                Err(GitError::Conflict { files: Vec::new() }.into())
            } else {
                Err(GitError::RebaseFailed(stderr.to_string()).into())
            }
        }
    }

    /// Abort an in-progress rebase.
    pub fn abort_rebase(&self) -> Result<()> {
        let workdir = self
            .workdir()
            .ok_or_else(|| GitError::RepositoryNotFound("No working directory".to_string()))?;

        std::process::Command::new("git")
            .args(["rebase", "--abort"])
            .current_dir(workdir)
            .output()
            .map_err(|e| GitError::RebaseFailed(e.to_string()))?;

        Ok(())
    }

    /// Continue an in-progress rebase after conflicts are resolved.
    pub fn continue_rebase(&self) -> Result<()> {
        let workdir = self
            .workdir()
            .ok_or_else(|| GitError::RepositoryNotFound("No working directory".to_string()))?;

        let output = std::process::Command::new("git")
            .args(["rebase", "--continue"])
            .current_dir(workdir)
            .output()
            .map_err(|e| GitError::RebaseFailed(e.to_string()))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("conflict") || stderr.contains("CONFLICT") {
                Err(GitError::Conflict { files: Vec::new() }.into())
            } else {
                Err(GitError::RebaseFailed(stderr.to_string()).into())
            }
        }
    }

    /// Check if a rebase is in progress.
    pub fn is_rebase_in_progress(&self) -> bool {
        let workdir = match self.workdir() {
            Some(w) => w,
            None => return false,
        };

        let rebase_merge = workdir.join(".git").join("rebase-merge");
        let rebase_apply = workdir.join(".git").join("rebase-apply");

        rebase_merge.exists() || rebase_apply.exists()
    }

    /// Get the state of an in-progress rebase.
    pub fn get_rebase_state(&self) -> Option<RebaseState> {
        let workdir = self.workdir()?;
        let git_dir = workdir.join(".git");

        // Check for rebase-merge (interactive rebase)
        let rebase_merge = git_dir.join("rebase-merge");
        if rebase_merge.exists() {
            let onto = std::fs::read_to_string(rebase_merge.join("onto"))
                .ok()
                .map(|s| s.trim().to_string());
            let head_name = std::fs::read_to_string(rebase_merge.join("head-name"))
                .ok()
                .map(|s| s.trim().replace("refs/heads/", ""));

            return Some(RebaseState {
                branch: head_name,
                onto,
            });
        }

        // Check for rebase-apply (am-style rebase)
        let rebase_apply = git_dir.join("rebase-apply");
        if rebase_apply.exists() {
            let onto = std::fs::read_to_string(rebase_apply.join("onto"))
                .ok()
                .map(|s| s.trim().to_string());
            let head_name = std::fs::read_to_string(rebase_apply.join("head-name"))
                .ok()
                .map(|s| s.trim().replace("refs/heads/", ""));

            return Some(RebaseState {
                branch: head_name,
                onto,
            });
        }

        None
    }

    /// Count commits that would be replayed when rebasing branch onto target.
    /// Returns the number of commits unique to branch that are not in target.
    pub fn commits_to_replay(&self, branch: &str, target: &str) -> Result<i32> {
        let branch_oid = self
            .repo
            .revparse_single(&format!("refs/heads/{}", branch))
            .map_err(|e| GitError::Branch(format!("Branch not found: {}", e)))?
            .id();

        let target_oid = self
            .repo
            .revparse_single(&format!("refs/heads/{}", target))
            .map_err(|e| GitError::Branch(format!("Target branch not found: {}", e)))?
            .id();

        // Find merge base
        let merge_base = self
            .repo
            .merge_base(branch_oid, target_oid)
            .map_err(|e| GitError::Branch(format!("Failed to find merge base: {}", e)))?;

        // Count commits from merge base to branch head
        let mut count = 0;
        let mut revwalk = self
            .repo
            .revwalk()
            .map_err(|e| GitError::Branch(e.to_string()))?;
        revwalk
            .push(branch_oid)
            .map_err(|e| GitError::Branch(e.to_string()))?;
        revwalk
            .hide(merge_base)
            .map_err(|e| GitError::Branch(e.to_string()))?;

        for _ in revwalk {
            count += 1;
        }

        Ok(count)
    }

    /// Get files with conflicts.
    pub fn get_conflict_files(&self) -> Vec<String> {
        let workdir = match self.workdir() {
            Some(w) => w,
            None => return Vec::new(),
        };

        // Use git status to find conflicting files
        let output = std::process::Command::new("git")
            .args(["status", "--porcelain=v2"])
            .current_dir(workdir)
            .output()
            .ok();

        if let Some(output) = output {
            if output.status.success() {
                return String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter(|line| line.starts_with("u ")) // Unmerged files
                    .filter_map(|line| line.split_whitespace().last())
                    .map(|s| s.to_string())
                    .collect();
            }
        }

        Vec::new()
    }

    /// Force push a branch to remote.
    pub fn force_push(&self, branch: &str, remote: &str) -> Result<()> {
        let workdir = self
            .workdir()
            .ok_or_else(|| GitError::RepositoryNotFound("No working directory".to_string()))?;

        let output = std::process::Command::new("git")
            .args(["push", "--force-with-lease", remote, branch])
            .current_dir(workdir)
            .output()
            .map_err(|e| GitError::Remote(e.to_string()))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(GitError::Remote(stderr.to_string()).into())
        }
    }
}

impl GitOperations for Git2Backend {
    fn checkout_branch(&self, branch: &str) -> Result<()> {
        let (object, reference) = self
            .repo
            .revparse_ext(&format!("refs/heads/{}", branch))
            .map_err(|e| GitError::Branch(format!("Branch not found: {}", e)))?;

        self.repo
            .checkout_tree(&object, None)
            .map_err(|e| GitError::Branch(format!("Failed to checkout: {}", e)))?;

        if let Some(reference) = reference {
            self.repo
                .set_head(reference.name().unwrap_or("HEAD"))
                .map_err(|e| GitError::Branch(format!("Failed to set HEAD: {}", e)))?;
        }

        Ok(())
    }

    fn create_branch(&self, name: &str, from: &str) -> Result<()> {
        let base_commit = self
            .repo
            .revparse_single(&format!("refs/heads/{}", from))
            .map_err(|e| GitError::Branch(format!("Base branch not found: {}", e)))?
            .peel_to_commit()
            .map_err(|e| GitError::Branch(format!("Failed to get commit: {}", e)))?;

        self.repo
            .branch(name, &base_commit, false)
            .map_err(|e| GitError::Branch(format!("Failed to create branch: {}", e)))?;

        Ok(())
    }

    fn current_branch(&self) -> Result<String> {
        let head = self
            .repo
            .head()
            .map_err(|e| GitError::Branch(format!("Failed to get HEAD: {}", e)))?;

        let name = head
            .shorthand()
            .ok_or_else(|| GitError::Branch("HEAD is not a branch".to_string()))?;

        Ok(name.to_string())
    }

    fn fetch(&self, remote: &str) -> Result<()> {
        let mut remote = self
            .repo
            .find_remote(remote)
            .map_err(|e| GitError::Remote(format!("Remote not found: {}", e)))?;

        remote
            .fetch(&[] as &[&str], None, None)
            .map_err(|e| GitError::Remote(format!("Fetch failed: {}", e)))?;

        Ok(())
    }

    fn branch_exists(&self, name: &str) -> Result<bool> {
        Ok(self.repo.find_branch(name, git2::BranchType::Local).is_ok())
    }

    fn default_branch(&self) -> Result<String> {
        // Try common default branch names
        for name in ["main", "master"] {
            if self.branch_exists(name)? {
                return Ok(name.to_string());
            }
        }

        // Fall back to trying to get default from origin
        if let Ok(remote) = self.repo.find_remote("origin") {
            if let Ok(default_ref) = remote.default_branch() {
                let name = std::str::from_utf8(&default_ref).unwrap_or("");
                let name = name.strip_prefix("refs/heads/").unwrap_or(name);
                if !name.is_empty() {
                    return Ok(name.to_string());
                }
            }
        }

        Err(GitError::Branch("Could not determine default branch".to_string()).into())
    }
}

/// CLI-based fallback for complex Git operations.
pub struct CliBackend {
    repo_path: PathBuf,
}

impl CliBackend {
    /// Create a new CLI backend for the given repository path.
    pub fn new(repo_path: PathBuf) -> Self {
        Self { repo_path }
    }

    /// Run a git command.
    fn run(&self, args: &[&str]) -> Result<String> {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::Git2(git2::Error::from_str(&e.to_string())))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(GitError::Branch(stderr.to_string()).into())
        }
    }
}

impl GitOperations for CliBackend {
    fn checkout_branch(&self, branch: &str) -> Result<()> {
        self.run(&["checkout", branch])?;
        Ok(())
    }

    fn create_branch(&self, name: &str, from: &str) -> Result<()> {
        self.run(&["checkout", "-b", name, from])?;
        Ok(())
    }

    fn current_branch(&self) -> Result<String> {
        self.run(&["rev-parse", "--abbrev-ref", "HEAD"])
    }

    fn fetch(&self, remote: &str) -> Result<()> {
        self.run(&["fetch", remote])?;
        Ok(())
    }

    fn branch_exists(&self, name: &str) -> Result<bool> {
        Ok(self
            .run(&["rev-parse", "--verify", &format!("refs/heads/{}", name)])
            .is_ok())
    }

    fn default_branch(&self) -> Result<String> {
        // Try to get from remote
        if let Ok(output) = self.run(&["symbolic-ref", "refs/remotes/origin/HEAD"]) {
            if let Some(branch) = output.strip_prefix("refs/remotes/origin/") {
                return Ok(branch.to_string());
            }
        }

        // Fall back to checking common names
        for name in ["main", "master"] {
            if self.branch_exists(name)? {
                return Ok(name.to_string());
            }
        }

        Err(GitError::Branch("Could not determine default branch".to_string()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn init_test_repo() -> (tempfile::TempDir, git2::Repository) {
        let dir = tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();

        // Create initial commit - use a block to drop the tree before we return
        {
            let sig = git2::Signature::now("Test", "test@example.com").unwrap();
            let tree_id = repo.index().unwrap().write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
                .unwrap();
        }

        // Rename master to main if needed
        {
            let head = repo.head().unwrap();
            if head.shorthand() == Some("master") {
                let mut branch = repo.find_branch("master", git2::BranchType::Local).unwrap();
                branch.rename("main", true).unwrap();
            }
        }

        (dir, repo)
    }

    #[test]
    fn test_git2_current_branch() {
        let (dir, _repo) = init_test_repo();
        let backend = Git2Backend::discover(dir.path()).unwrap();
        let branch = backend.current_branch().unwrap();
        assert!(branch == "main" || branch == "master");
    }

    #[test]
    fn test_git2_create_branch() {
        let (dir, _repo) = init_test_repo();
        let backend = Git2Backend::discover(dir.path()).unwrap();
        let current = backend.current_branch().unwrap();

        backend.create_branch("feature-test", &current).unwrap();
        assert!(backend.branch_exists("feature-test").unwrap());
    }

    #[test]
    fn test_git2_checkout_branch() {
        let (dir, _repo) = init_test_repo();
        let backend = Git2Backend::discover(dir.path()).unwrap();
        let current = backend.current_branch().unwrap();

        backend.create_branch("feature-test", &current).unwrap();
        backend.checkout_branch("feature-test").unwrap();
        assert_eq!(backend.current_branch().unwrap(), "feature-test");
    }
}
