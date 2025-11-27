//! Git operations module.
//!
//! This module provides abstractions for Git operations using git2 (libgit2)
//! with a CLI fallback for complex operations.

use crate::error::{GitError, Result};
use std::path::{Path, PathBuf};

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
        Ok(self
            .repo
            .find_branch(name, git2::BranchType::Local)
            .is_ok())
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
        Ok(self.run(&["rev-parse", "--verify", &format!("refs/heads/{}", name)]).is_ok())
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
