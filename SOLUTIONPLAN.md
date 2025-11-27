# Solution Plan: Rust-Based Desktop Git Client

## Document Overview

This document describes the high-level architecture and implementation plan for a cross-platform desktop Git client built in Rust, featuring a Tower-style PR dashboard and Graphite-style stacked PR workflow.

---

## 1. System Overview Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              PRESENTATION LAYER                              │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                         Tauri + Web UI (React/Svelte)                   ││
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐ ││
│  │  │  PR Dashboard │  │ Stack View   │  │ Diff Viewer  │  │  Settings   │ ││
│  │  │    View       │  │              │  │              │  │    View     │ ││
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └─────────────┘ ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                      │                                       │
│                              Tauri Commands (IPC)                            │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                               DOMAIN LAYER (Rust)                            │
│  ┌───────────────────┐  ┌───────────────────┐  ┌───────────────────────┐   │
│  │   PR Service      │  │  Stack Service    │  │    Sync Service       │   │
│  │  - List PRs       │  │  - Stack CRUD     │  │  - Background sync    │   │
│  │  - PR Details     │  │  - Restack logic  │  │  - Cache management   │   │
│  │  - PR Actions     │  │  - Conflict detect│  │  - Rate limiting      │   │
│  └───────────────────┘  └───────────────────┘  └───────────────────────┘   │
│  ┌───────────────────┐  ┌───────────────────┐  ┌───────────────────────┐   │
│  │   Auth Service    │  │  Repo Service     │  │   Settings Service    │   │
│  │  - OAuth flow     │  │  - Repo detection │  │  - User preferences   │   │
│  │  - Token storage  │  │  - Remote parsing │  │  - Theme, intervals   │   │
│  └───────────────────┘  └───────────────────┘  └───────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              DATA LAYER (Rust)                               │
│  ┌─────────────────────────┐              ┌─────────────────────────────┐  │
│  │    GitHub GraphQL       │              │       Git Operations        │  │
│  │      Client             │              │                             │  │
│  │  ┌─────────────────┐   │              │  ┌───────────────────────┐  │  │
│  │  │ graphql-client  │   │              │  │   git2 (libgit2)      │  │  │
│  │  │ + reqwest       │   │              │  │   - Branch ops        │  │  │
│  │  └─────────────────┘   │              │  │   - Rebase            │  │  │
│  │  - PR queries          │              │  │   - Checkout          │  │  │
│  │  - PR mutations        │              │  │   - Commit inspection │  │  │
│  │  - Pagination          │              │  └───────────────────────┘  │  │
│  │  - Rate limiting       │              │                             │  │
│  └─────────────────────────┘              └─────────────────────────────┘  │
│                                                                             │
│  ┌─────────────────────────┐              ┌─────────────────────────────┐  │
│  │    Local Cache          │              │    Secure Storage           │  │
│  │    (SQLite + rusqlite)  │              │    (keyring crate)          │  │
│  │  - PR metadata          │              │  - GitHub tokens            │  │
│  │  - Stack definitions    │              │  - User credentials         │  │
│  └─────────────────────────┘              └─────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                            EXTERNAL SYSTEMS                                  │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐ │
│  │   GitHub GraphQL    │  │   Local Git Repo    │  │   OS Keychain       │ │
│  │   API (v4)          │  │   (.git directory)  │  │                     │ │
│  └─────────────────────┘  └─────────────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Component/Module Breakdown

### 2.1 Presentation Layer

| Module | Responsibility | Technology |
|--------|---------------|------------|
| `ui/` | Web-based UI components | TypeScript + React or Svelte |
| `ui/views/pr-dashboard` | PR list and detail views | React components |
| `ui/views/stack-view` | Stack visualization and management | React + D3 or custom SVG |
| `ui/views/diff-viewer` | Unified diff display | Monaco Editor or custom |
| `ui/views/settings` | User preferences and auth | React components |
| `ui/state/` | Frontend state management | Zustand or Redux Toolkit |

### 2.2 Domain Layer (Rust - `src-tauri/src/`)

| Module | Responsibility | Key Types |
|--------|---------------|-----------|
| `domain/pr/` | Pull request business logic | `PullRequest`, `PRService` |
| `domain/stack/` | Stack management and restacking | `Stack`, `StackBranch`, `StackService` |
| `domain/auth/` | Authentication flows | `AuthService`, `AuthState` |
| `domain/repo/` | Repository detection and management | `Repository`, `RepoService` |
| `domain/sync/` | Background synchronization | `SyncService`, `SyncState` |
| `commands/` | Tauri command handlers | IPC bridge functions |

### 2.3 Data Layer (Rust - `src-tauri/src/`)

| Module | Responsibility | Key Crates |
|--------|---------------|------------|
| `github/` | GitHub GraphQL API client | `graphql-client`, `reqwest` |
| `github/queries/` | GraphQL query definitions | Generated from `.graphql` files |
| `git/` | Local Git operations | `git2` |
| `cache/` | Local data persistence | `rusqlite`, `serde` |
| `keyring/` | Secure credential storage | `keyring` |

### 2.4 Shared/Infrastructure

| Module | Responsibility | Key Crates |
|--------|---------------|------------|
| `error/` | Error types and handling | `thiserror`, `anyhow` |
| `config/` | Configuration management | `serde`, `toml` |
| `logging/` | Application logging | `tracing`, `tracing-subscriber` |

---

## 3. Key Rust Crates

### Core Application
| Crate | Purpose | Version |
|-------|---------|---------|
| `tauri` | Desktop application framework | ^2.0 |
| `tokio` | Async runtime | ^1.0 |
| `serde` | Serialization/deserialization | ^1.0 |
| `serde_json` | JSON handling | ^1.0 |

### GitHub Integration
| Crate | Purpose | Version |
|-------|---------|---------|
| `graphql-client` | GraphQL query generation | ^0.14 |
| `reqwest` | HTTP client | ^0.12 |
| `octocrab` | GitHub REST API (supplementary) | ^0.38 |

### Git Operations
| Crate | Purpose | Version |
|-------|---------|---------|
| `git2` | libgit2 bindings | ^0.18 |

### Storage & Security
| Crate | Purpose | Version |
|-------|---------|---------|
| `rusqlite` | SQLite database | ^0.31 |
| `keyring` | OS keychain access | ^2.0 |

### Utilities
| Crate | Purpose | Version |
|-------|---------|---------|
| `thiserror` | Error derive macros | ^1.0 |
| `anyhow` | Error handling | ^1.0 |
| `tracing` | Structured logging | ^0.1 |
| `chrono` | Date/time handling | ^0.4 |
| `url` | URL parsing | ^2.0 |

---

## 4. GitHub GraphQL Integration

### 4.1 Core Queries

#### List Pull Requests
```graphql
query ListPullRequests($owner: String!, $repo: String!, $baseRefName: String, $first: Int!, $after: String) {
  repository(owner: $owner, name: $repo) {
    pullRequests(
      baseRefName: $baseRefName
      states: [OPEN]
      first: $first
      after: $after
      orderBy: { field: UPDATED_AT, direction: DESC }
    ) {
      pageInfo {
        hasNextPage
        endCursor
      }
      nodes {
        number
        title
        state
        isDraft
        createdAt
        updatedAt
        author {
          login
          avatarUrl
        }
        labels(first: 10) {
          nodes {
            name
            color
          }
        }
        reviewDecision
        headRefName
        baseRefName
        mergeable
        commits {
          totalCount
        }
      }
    }
  }
}
```

#### Get Pull Request Details
```graphql
query GetPullRequestDetails($owner: String!, $repo: String!, $number: Int!) {
  repository(owner: $owner, name: $repo) {
    pullRequest(number: $number) {
      number
      title
      body
      state
      isDraft
      mergeable
      mergeStateStatus
      author {
        login
        avatarUrl
      }
      headRefName
      baseRefName
      headRefOid
      baseRefOid
      additions
      deletions
      changedFiles
      commits(first: 100) {
        nodes {
          commit {
            oid
            message
            author {
              name
              date
            }
          }
        }
      }
      files(first: 100) {
        nodes {
          path
          additions
          deletions
          changeType
        }
      }
      reviews(first: 50) {
        nodes {
          author {
            login
          }
          state
          submittedAt
        }
      }
      reviewRequests(first: 10) {
        nodes {
          requestedReviewer {
            ... on User {
              login
            }
          }
        }
      }
    }
  }
}
```

### 4.2 Core Mutations

#### Create Pull Request
```graphql
mutation CreatePullRequest($input: CreatePullRequestInput!) {
  createPullRequest(input: $input) {
    pullRequest {
      number
      url
    }
  }
}
```

#### Update Pull Request Base
```graphql
mutation UpdatePullRequestBranch($pullRequestId: ID!, $expectedHeadOid: GitObjectID) {
  updatePullRequestBranch(input: { 
    pullRequestId: $pullRequestId
    expectedHeadOid: $expectedHeadOid
  }) {
    pullRequest {
      number
      headRefOid
    }
  }
}
```

#### Merge Pull Request
```graphql
mutation MergePullRequest($pullRequestId: ID!, $mergeMethod: PullRequestMergeMethod) {
  mergePullRequest(input: { 
    pullRequestId: $pullRequestId
    mergeMethod: $mergeMethod
  }) {
    pullRequest {
      number
      merged
    }
  }
}
```

#### Close Pull Request
```graphql
mutation ClosePullRequest($pullRequestId: ID!) {
  closePullRequest(input: { pullRequestId: $pullRequestId }) {
    pullRequest {
      number
      state
    }
  }
}
```

### 4.3 GraphQL Client Architecture (Rust)

```rust
// src-tauri/src/github/client.rs

pub struct GitHubClient {
    http: reqwest::Client,
    token: String,
    rate_limiter: RateLimiter,
}

impl GitHubClient {
    pub async fn query<Q: GraphQLQuery>(&self, variables: Q::Variables) -> Result<Q::ResponseData> {
        self.rate_limiter.acquire().await?;
        
        let body = Q::build_query(variables);
        let response = self.http
            .post("https://api.github.com/graphql")
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await?;
            
        self.handle_rate_limit_headers(&response);
        
        let data: Response<Q::ResponseData> = response.json().await?;
        
        if let Some(errors) = data.errors {
            return Err(GitHubError::GraphQL(errors).into());
        }
        
        data.data.ok_or_else(|| GitHubError::EmptyResponse.into())
    }
}

// Pagination helper
pub async fn paginate<Q, T, F>(
    client: &GitHubClient,
    initial_vars: Q::Variables,
    extract: F,
) -> Result<Vec<T>>
where
    Q: GraphQLQuery,
    F: Fn(&Q::ResponseData) -> (Vec<T>, Option<String>),
{
    let mut all_items = Vec::new();
    let mut cursor: Option<String> = None;
    
    loop {
        let vars = update_cursor(initial_vars.clone(), cursor);
        let response = client.query::<Q>(vars).await?;
        let (items, next_cursor) = extract(&response);
        
        all_items.extend(items);
        
        match next_cursor {
            Some(c) => cursor = Some(c),
            None => break,
        }
    }
    
    Ok(all_items)
}
```

---

## 5. Signature Flow Sequences

### 5.1 Flow: Review and Merge a PR in the Dashboard

```
┌────────┐     ┌─────────┐     ┌──────────┐     ┌───────────┐     ┌────────┐
│  User  │     │   UI    │     │ PRService│     │GitHubClient    │ GitHub │
└───┬────┘     └────┬────┘     └────┬─────┘     └─────┬─────┘     └───┬────┘
    │               │               │                 │               │
    │ Open PR List  │               │                 │               │
    ├──────────────►│               │                 │               │
    │               │ list_prs()    │                 │               │
    │               ├──────────────►│                 │               │
    │               │               │ query(ListPRs)  │               │
    │               │               ├────────────────►│               │
    │               │               │                 │ GraphQL Query │
    │               │               │                 ├──────────────►│
    │               │               │                 │◄──────────────┤
    │               │               │◄────────────────┤               │
    │               │◄──────────────┤                 │               │
    │◄──────────────┤ Render PR List│                 │               │
    │               │               │                 │               │
    │ Select PR #42 │               │                 │               │
    ├──────────────►│               │                 │               │
    │               │ get_pr(42)    │                 │               │
    │               ├──────────────►│                 │               │
    │               │               │ query(GetPRDetails)             │
    │               │               ├────────────────►│               │
    │               │               │                 ├──────────────►│
    │               │               │                 │◄──────────────┤
    │               │               │◄────────────────┤               │
    │               │◄──────────────┤                 │               │
    │◄──────────────┤ Render Detail │                 │               │
    │               │               │                 │               │
    │ Click "Checkout" │            │                 │               │
    ├──────────────►│               │                 │               │
    │               │ checkout_pr(42)                 │               │
    │               ├──────────────►│                 │               │
    │               │               │ git2::checkout()│               │
    │               │               ├─────────────────┼───┐           │
    │               │               │                 │   │ Local Git │
    │               │               │◄────────────────┼───┘           │
    │               │◄──────────────┤                 │               │
    │◄──────────────┤ Branch switched                 │               │
    │               │               │                 │               │
    │ Click "Merge" │               │                 │               │
    ├──────────────►│               │                 │               │
    │               │ Show merge options              │               │
    │               │◄──────────────┤                 │               │
    │ Select "Squash"               │                 │               │
    ├──────────────►│               │                 │               │
    │               │ merge_pr(42, SQUASH)            │               │
    │               ├──────────────►│                 │               │
    │               │               │ mutate(MergePR) │               │
    │               │               ├────────────────►│               │
    │               │               │                 ├──────────────►│
    │               │               │                 │◄──────────────┤
    │               │               │◄────────────────┤               │
    │               │◄──────────────┤                 │               │
    │◄──────────────┤ PR Merged ✓   │                 │               │
    │               │               │                 │               │
```

**Step-by-Step:**

1. **User opens PR Dashboard**
   - UI component mounts and calls `PRService.list_prs(base_branch: "main")`
   - PRService checks local cache, returns cached data if fresh
   - PRService triggers background refresh via GitHubClient
   - UI renders PR list with metadata

2. **User selects PR #42**
   - UI calls `PRService.get_pr_details(42)`
   - PRService fetches full PR data including commits, files, reviews
   - UI renders detail view with diff, commits, review status

3. **User clicks "Checkout"**
   - UI calls `PRService.checkout_pr(42)`
   - PRService uses git2 to fetch and checkout the PR branch
   - UI updates current branch indicator

4. **User reviews code locally** (manual step)

5. **User clicks "Merge"**
   - UI shows merge strategy options (merge, squash, rebase)
   - User selects "Squash and merge"
   - UI calls `PRService.merge_pr(42, MergeMethod::Squash)`
   - PRService calls GitHub GraphQL mutation
   - UI shows success and removes PR from list

---

### 5.2 Flow: Create and Maintain a Stack of PRs with Automatic Restacking

```
┌────────┐     ┌─────────┐     ┌────────────┐     ┌───────────┐     ┌────────┐
│  User  │     │   UI    │     │StackService│     │GitHubClient    │  Git   │
└───┬────┘     └────┬────┘     └─────┬──────┘     └─────┬─────┘     └───┬────┘
    │               │                │                  │               │
    │ Create new branch from main    │                  │               │
    ├──────────────►│                │                  │               │
    │               │ create_stack_branch("feature-a", "main")         │
    │               ├───────────────►│                  │               │
    │               │                │ git2::create_branch             │
    │               │                ├──────────────────┼──────────────►│
    │               │                │ persist_stack_metadata          │
    │               │                ├──────────────────┼───┐           │
    │               │                │◄─────────────────┼───┘           │
    │               │◄───────────────┤                  │               │
    │◄──────────────┤ Stack: [main] → feature-a        │               │
    │               │                │                  │               │
    │ ... make commits ...           │                  │               │
    │               │                │                  │               │
    │ Create branch from feature-a   │                  │               │
    ├──────────────►│                │                  │               │
    │               │ create_stack_branch("feature-b", "feature-a")    │
    │               ├───────────────►│                  │               │
    │               │                │ ... (same as above)             │
    │               │◄───────────────┤                  │               │
    │◄──────────────┤ Stack: [main] → feature-a → feature-b            │
    │               │                │                  │               │
    │ Create PRs    │                │                  │               │
    ├──────────────►│                │                  │               │
    │               │ create_stack_prs()               │               │
    │               ├───────────────►│                  │               │
    │               │                │ For each branch in stack:       │
    │               │                │   mutate(CreatePR)              │
    │               │                ├─────────────────►│               │
    │               │                │                  ├──────────────►│
    │               │                │                  │◄──────────────┤
    │               │                │◄─────────────────┤               │
    │               │◄───────────────┤                  │               │
    │◄──────────────┤ PRs created: feature-a→main, feature-b→feature-a │
    │               │                │                  │               │
    │ === Time passes, feature-a merges ===            │               │
    │               │                │                  │               │
    │               │ [Background Sync detects merge]  │               │
    │               │                │ query(ListPRs)  │               │
    │               │                ├─────────────────►│               │
    │               │                │                  ├──────────────►│
    │               │                │                  │◄──────────────┤
    │               │                │◄─────────────────┤               │
    │               │                │ detect_merge(feature-a)         │
    │               │◄───────────────┤                  │               │
    │◄──────────────┤ Stack needs restack!             │               │
    │               │                │                  │               │
    │ Click "Restack"                │                  │               │
    ├──────────────►│                │                  │               │
    │               │ restack()      │                  │               │
    │               ├───────────────►│                  │               │
    │               │                │ 1. git2::fetch("origin/main")   │
    │               │                ├──────────────────┼──────────────►│
    │               │                │ 2. git2::rebase(feature-b, main)│
    │               │                ├──────────────────┼──────────────►│
    │               │                │ 3. git2::force_push(feature-b)  │
    │               │                ├──────────────────┼──────────────►│
    │               │                │ 4. mutate(UpdatePRBase)         │
    │               │                ├─────────────────►│               │
    │               │                │                  ├──────────────►│
    │               │                │                  │◄──────────────┤
    │               │                │◄─────────────────┤               │
    │               │                │ 5. update_stack_metadata        │
    │               │                ├──────────────────┼───┐           │
    │               │                │◄─────────────────┼───┘           │
    │               │◄───────────────┤                  │               │
    │◄──────────────┤ Stack restacked: [main] → feature-b              │
    │               │                │                  │               │
```

**Step-by-Step:**

1. **User creates first stack branch**
   - User initiates "New Stack Branch" from main
   - StackService creates branch using git2
   - StackService persists parent relationship to local metadata (`.git/stack-metadata.json`)
   - UI updates stack visualization

2. **User creates second branch on stack**
   - User creates feature-b with feature-a as parent
   - StackService adds to stack metadata
   - Stack is now: main → feature-a → feature-b

3. **User creates PRs for stack**
   - StackService creates PR for feature-a targeting main
   - StackService creates PR for feature-b targeting feature-a
   - Each PR description includes stack context
   - UI shows PRs linked to stack branches

4. **Parent PR merges (detected by background sync)**
   - SyncService polls GitHub for PR status changes
   - Detects feature-a PR has merged
   - Notifies UI that stack needs restacking

5. **User initiates restack**
   - User clicks "Restack" button
   - StackService fetches latest main from origin
   - StackService rebases feature-b onto main (since feature-a is now in main)
   - If conflicts occur:
     - StackService pauses and surfaces conflict to UI
     - User resolves conflicts manually
     - User clicks "Continue Restack"
   - StackService force-pushes feature-b
   - StackService updates PR via GraphQL to target main instead of feature-a
   - StackService updates local metadata
   - UI reflects updated stack: main → feature-b

---

## 6. Stack Metadata Persistence

### 6.1 Storage Location

Stack metadata is stored in `.git/stack-metadata.json` (not tracked in version control).

### 6.2 Schema

```json
{
  "version": 1,
  "stacks": [
    {
      "id": "uuid-1",
      "root": "main",
      "branches": [
        {
          "name": "feature-a",
          "parent": "main",
          "pr_number": 42,
          "created_at": "2025-11-27T00:00:00Z"
        },
        {
          "name": "feature-b",
          "parent": "feature-a",
          "pr_number": 43,
          "created_at": "2025-11-27T01:00:00Z"
        }
      ]
    }
  ],
  "last_sync": "2025-11-27T02:00:00Z"
}
```

### 6.3 Robustness Against External Git Operations

| Scenario | Detection | Resolution |
|----------|-----------|------------|
| Branch deleted externally | Branch ref doesn't exist | Mark as orphaned, prompt user |
| Branch reset externally | Commit SHA mismatch | Warn user, offer to update metadata |
| Branches rebased externally | Parent commit not in history | Recalculate parent relationships |
| New commits added externally | Head SHA changed | Normal operation, continue |
| Branch renamed externally | Branch ref doesn't exist | Mark as orphaned, prompt user |

### 6.4 Reconciliation Process

```rust
pub fn reconcile_stack(&mut self, repo: &git2::Repository) -> Result<ReconcileReport> {
    let mut report = ReconcileReport::new();
    
    for stack in &mut self.stacks {
        for branch in &mut stack.branches {
            // Check if branch still exists
            match repo.find_branch(&branch.name, BranchType::Local) {
                Ok(git_branch) => {
                    // Check if parent relationship is still valid
                    if !self.is_ancestor(repo, &branch.parent, &branch.name)? {
                        report.add_warning(
                            branch.name.clone(),
                            Warning::ParentNotAncestor
                        );
                    }
                }
                Err(_) => {
                    branch.status = BranchStatus::Orphaned;
                    report.add_orphan(branch.name.clone());
                }
            }
        }
    }
    
    Ok(report)
}
```

---

## 7. Data Synchronization Strategy

### 7.1 Approach: Polling with Smart Refresh

| Method | Pros | Cons | Decision |
|--------|------|------|----------|
| Polling | Simple, works offline-first, no server requirements | Latency, rate limit pressure | **Selected** |
| Webhooks | Real-time updates | Requires server, firewall issues | Future |
| Manual refresh | User control | Poor UX | Supplementary |

### 7.2 Implementation

```rust
pub struct SyncService {
    interval: Duration,
    github: Arc<GitHubClient>,
    cache: Arc<Cache>,
    state: Arc<RwLock<SyncState>>,
}

impl SyncService {
    pub async fn start(&self) {
        let mut interval = tokio::time::interval(self.interval);
        
        loop {
            interval.tick().await;
            
            if self.should_sync().await {
                match self.sync().await {
                    Ok(_) => self.emit_update().await,
                    Err(e) => self.handle_error(e).await,
                }
            }
        }
    }
    
    async fn sync(&self) -> Result<()> {
        // 1. Fetch latest PR list
        let prs = self.github.list_prs(&self.repo).await?;
        
        // 2. Diff with cache
        let changes = self.cache.diff(&prs)?;
        
        // 3. Update cache
        self.cache.update(prs)?;
        
        // 4. Emit change events for UI
        for change in changes {
            self.emit_event(SyncEvent::PRChanged(change)).await;
        }
        
        Ok(())
    }
}
```

### 7.3 Rate Limiting

- GitHub GraphQL: 5,000 points/hour
- Track remaining budget from response headers
- Implement exponential backoff on rate limit errors
- Adjust polling interval based on remaining budget

---

## 8. Technical Risks and Open Questions

### 8.1 Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Complex merge conflicts in restacking | High | Medium | Provide clear conflict UI, allow abort |
| Git2 rebase complexity | Medium | High | Consider shelling out for complex rebases |
| GitHub API rate limiting | Medium | Medium | Smart caching, adjust polling |
| Cross-platform secure storage issues | Low | Medium | Fallback to encrypted file storage |
| Tauri WebView inconsistencies | Medium | Medium | Extensive cross-platform testing |

### 8.2 Open Questions for Next Iteration

1. **Conflict Resolution UI**: What level of conflict resolution should be built into v1 vs. deferring to external tools?

2. **Multi-repo Workspaces**: Should v1 support multiple repositories in a single window?

3. **Offline Mode**: How much functionality should work offline? Just viewing cached data?

4. **PR Templates**: Should the client support repo-specific PR templates?

5. **Branch Naming**: Should the client enforce or suggest naming conventions for stacked branches?

6. **Merge Queue Integration**: GitHub's merge queue feature - integrate in v1?

7. **Draft PR Workflow**: Automatic draft→ready transitions when stack is ready?

---

## 9. UI Framework Decision Summary

**Recommendation: Tauri + Web UI (React or Svelte)**

| Criteria | Tauri + Web | Pure egui |
|----------|-------------|-----------|
| Cross-platform consistency | Excellent | Good |
| Developer pool | Large (web devs) | Small (Rust-only) |
| UI polish achievable | High | Medium |
| Bundle size | ~5-10MB | ~2-5MB |
| Performance | Good | Excellent |
| Component ecosystem | Massive | Limited |
| Accessibility | Mature | Developing |

See ADR-001 for detailed justification.

---

## 10. Git Integration Decision Summary

**Recommendation: git2 (libgit2) as primary, CLI fallback for edge cases**

| Criteria | git2 | Git CLI |
|----------|------|---------|
| Performance | Faster for most ops | Subprocess overhead |
| Cross-platform | Bundled library | Requires Git installed |
| API stability | Stable Rust bindings | CLI output parsing fragile |
| Feature coverage | 90%+ of needs | 100% |
| Complex rebases | Limited control | Full control |

See ADR-002 for detailed justification.

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2025-11-27 | Architecture Team | Initial draft |
