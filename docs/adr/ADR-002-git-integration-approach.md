# ADR-002: Git Integration Approach

## Status

**Accepted**

## Context

The application needs to perform local Git operations including:

- Branch creation, checkout, and deletion
- Rebase operations (critical for stacked PRs)
- Fetching from remotes
- Force-pushing to update stacked PR branches
- Reading commit history and diffs
- Detecting merge conflicts

We must choose between:

1. **git2 (libgit2)**: Native Rust bindings to the libgit2 library
2. **Git CLI**: Shell out to the system's `git` command
3. **Hybrid**: Use git2 for most operations, CLI for complex cases

## Decision

We will use **git2 (libgit2) as the primary integration**, with **Git CLI as a fallback** for complex rebase operations and edge cases.

## Rationale

### Comparison Matrix

| Criterion | git2 (libgit2) | Git CLI |
|-----------|----------------|---------|
| **Cross-Platform Consistency** | ✅ Bundled library, no external dependency | ⚠️ Requires Git installed, version differences |
| **Performance** | ✅ In-process, no subprocess overhead | ⚠️ Process spawn for each operation |
| **Output Parsing** | ✅ Structured Rust types | ❌ Parse stdout/stderr strings |
| **Error Handling** | ✅ Typed errors | ⚠️ Exit codes + text parsing |
| **Feature Coverage** | ⚠️ ~90% of common operations | ✅ 100% of Git features |
| **Interactive Rebase** | ❌ Not supported directly | ✅ Full support |
| **Custom Merge Strategies** | ⚠️ Limited | ✅ Full support |
| **Worktree Operations** | ⚠️ Limited | ✅ Full support |

### Why git2 as Primary?

1. **No External Dependency**: Users don't need Git installed (though most developers have it)
2. **Consistent Behavior**: Same libgit2 version across all platforms
3. **Type Safety**: Operations return structured Rust types, not strings to parse
4. **Performance**: Batch operations without process spawn overhead
5. **Bundling**: Simplifies distribution and reduces "works on my machine" issues

### Why CLI Fallback?

1. **Complex Rebases**: git2's rebase API is lower-level and harder to use correctly
2. **Interactive Operations**: git2 doesn't support interactive rebase
3. **Edge Cases**: Some rare operations are easier via CLI
4. **Debugging**: Users can reproduce issues with standard Git commands

### Trade-offs Accepted

| Trade-off | Mitigation |
|-----------|------------|
| git2 rebase is complex | Implement carefully with extensive testing; fall back to CLI if needed |
| libgit2 not 100% Git-compatible | Test against Git CLI to ensure consistency for supported operations |
| Larger binary size (~2-5MB for libgit2) | Acceptable for a desktop application |
| Two code paths (git2 + CLI) | Abstract behind common trait; prefer git2, use CLI only when necessary |

## Consequences

### Positive

- Application works without Git installed
- Consistent behavior across platforms and Git versions
- Better error handling and type safety
- Faster batch operations

### Negative

- Must maintain two code paths for some operations
- libgit2 may lag behind Git features
- Larger binary size
- git2 learning curve for complex operations

### Neutral

- Need integration tests against real Git repos
- Should test parity between git2 and CLI paths

## Implementation Notes

### Abstraction Layer

```rust
// src-tauri/src/git/mod.rs

/// Trait abstracting Git operations
pub trait GitOperations {
    fn checkout_branch(&self, branch: &str) -> Result<()>;
    fn create_branch(&self, name: &str, from: &str) -> Result<()>;
    fn rebase(&self, branch: &str, onto: &str) -> Result<RebaseResult>;
    fn push(&self, branch: &str, force: bool) -> Result<()>;
    fn fetch(&self, remote: &str) -> Result<()>;
    // ... etc
}

/// Primary implementation using git2
pub struct Git2Backend {
    repo: git2::Repository,
}

/// Fallback implementation using CLI
pub struct CliBackend {
    repo_path: PathBuf,
}

/// Facade that tries git2 first, falls back to CLI
pub struct GitFacade {
    git2: Git2Backend,
    cli: CliBackend,
}

impl GitOperations for GitFacade {
    fn rebase(&self, branch: &str, onto: &str) -> Result<RebaseResult> {
        // Try git2 first
        match self.git2.rebase(branch, onto) {
            Ok(result) => Ok(result),
            Err(e) if self.should_fallback(&e) => {
                // Fall back to CLI for complex cases
                // Note: should_fallback() is a custom method that checks if
                // the error indicates a case where CLI would handle it better
                log::info!("Falling back to CLI for rebase: {}", e);
                self.cli.rebase(branch, onto)
            }
            Err(e) => Err(e),
        }
    }
}
```

### git2 Rebase Implementation

```rust
impl Git2Backend {
    pub fn rebase(&self, branch: &str, onto: &str) -> Result<RebaseResult> {
        let onto_ref = self.repo.find_reference(&format!("refs/heads/{}", onto))?;
        let onto_commit = onto_ref.peel_to_commit()?;
        
        let branch_ref = self.repo.find_reference(&format!("refs/heads/{}", branch))?;
        let branch_commit = branch_ref.peel_to_commit()?;
        
        let mut rebase = self.repo.rebase(
            Some(&branch_commit.as_object()),
            Some(&onto_commit.as_object()),
            None,
            Some(&mut RebaseOptions::new()),
        )?;
        
        let mut result = RebaseResult::new();
        
        while let Some(op) = rebase.next() {
            match op {
                Ok(operation) => {
                    // Apply the operation
                    let commit = self.repo.find_commit(operation.id())?;
                    
                    // Check for conflicts
                    let index = self.repo.index()?;
                    if index.has_conflicts() {
                        result.add_conflict(operation.id());
                        rebase.abort()?;
                        return Ok(result.with_status(RebaseStatus::Conflicts));
                    }
                    
                    // Commit the rebased change
                    rebase.commit(None, &commit.committer(), None)?;
                    result.add_applied(operation.id());
                }
                Err(e) => {
                    rebase.abort()?;
                    return Err(e.into());
                }
            }
        }
        
        rebase.finish(None)?;
        Ok(result.with_status(RebaseStatus::Success))
    }
}
```

### CLI Fallback for Complex Rebase

```rust
impl CliBackend {
    pub fn rebase(&self, branch: &str, onto: &str) -> Result<RebaseResult> {
        // First, checkout the branch
        self.run(&["checkout", branch])?;
        
        // Perform rebase
        let output = Command::new("git")
            .args(&["rebase", onto])
            .current_dir(&self.repo_path)
            .output()?;
        
        if output.status.success() {
            Ok(RebaseResult::success())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            if stderr.contains("CONFLICT") {
                // Abort the rebase to leave repo in clean state
                let _ = self.run(&["rebase", "--abort"]);
                Ok(RebaseResult::conflicts(self.parse_conflicts(&stderr)))
            } else {
                Err(GitError::RebaseFailed(stderr.to_string()))
            }
        }
    }
    
    fn run(&self, args: &[&str]) -> Result<Output> {
        Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(Into::into)
    }
}
```

### Testing Strategy

1. **Unit Tests**: Mock the `GitOperations` trait
2. **Integration Tests**: Use real Git repos (created in temp directories)
3. **Parity Tests**: Run same operation with git2 and CLI, compare results
4. **Edge Case Tests**: Test conflict scenarios, large repos, unusual branch structures

```rust
#[test]
fn test_rebase_parity() {
    let repo = create_test_repo();
    setup_branches(&repo, &["main", "feature-a", "feature-b"]);
    
    let git2_result = Git2Backend::new(&repo).rebase("feature-b", "main");
    
    // Reset repo
    reset_repo(&repo);
    
    // Note: In actual implementation, CliBackend::new() takes a PathBuf
    let cli_result = CliBackend::new(repo.path().to_path_buf()).rebase("feature-b", "main");
    
    assert_eq!(git2_result, cli_result);
}
```

## Alternatives Considered

### gitoxide (gix)

A pure Rust Git implementation. Currently not mature enough for production use in all required operations, but worth monitoring for future consideration.

### jj (Jujutsu)

A new VCS with Git compatibility. Interesting for future exploration but too novel for v1.

## References

- [git2 crate](https://docs.rs/git2)
- [libgit2 documentation](https://libgit2.org/)
- [gitoxide](https://github.com/Byron/gitoxide)
- [Tower's approach](https://www.git-tower.com/) - Uses libgit2 extensively

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2025-11-27 | Architecture Team | Initial decision |
