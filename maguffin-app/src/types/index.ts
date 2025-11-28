// TypeScript types matching the Rust domain types

export interface Author {
  login: string;
  avatar_url: string;
}

export interface Label {
  name: string;
  color: string;
}

export type PrState = "OPEN" | "CLOSED" | "MERGED";
export type ReviewDecision = "APPROVED" | "CHANGES_REQUESTED" | "REVIEW_REQUIRED";
export type Mergeable = "MERGEABLE" | "CONFLICTING" | "UNKNOWN";
export type MergeMethod = "MERGE" | "SQUASH" | "REBASE";

export interface PullRequest {
  // GraphQL node ID (required for mutations)
  id: string;
  number: number;
  title: string;
  body: string | null;
  state: PrState;
  is_draft: boolean;
  author: Author;
  head_ref: string;
  base_ref: string;
  labels: Label[];
  review_decision: ReviewDecision | null;
  mergeable: Mergeable;
  created_at: string;
  updated_at: string;
  commit_count: number;
  additions: number;
  deletions: number;
  changed_files: number;
}

export interface Commit {
  oid: string;
  message: string;
  author_name: string;
  authored_date: string;
}

export type ChangeType = "ADDED" | "DELETED" | "MODIFIED" | "RENAMED" | "COPIED" | "CHANGED";

export interface ChangedFile {
  path: string;
  additions: number;
  deletions: number;
  change_type: ChangeType;
}

export type ReviewState = "PENDING" | "COMMENTED" | "APPROVED" | "CHANGES_REQUESTED" | "DISMISSED";

export interface Review {
  author: string;
  state: ReviewState;
  submitted_at: string;
}

export interface PullRequestDetails {
  pr: PullRequest;
  commits: Commit[];
  files: ChangedFile[];
  reviews: Review[];
  review_requests: string[];
}

// Stack types
export type BranchStatus = "up_to_date" | "needs_rebase" | "conflicted" | "orphaned" | "unknown";

export interface StackBranch {
  name: string;
  parent: string;
  pr_number: number | null;
  status: BranchStatus;
  created_at: string;
  head_sha: string | null;
}

export interface Stack {
  id: string;
  root: string;
  branches: StackBranch[];
  created_at: string;
  updated_at: string;
}

// Auth types
export interface DeviceFlowPending {
  user_code: string;
  verification_uri: string;
  expires_at: string;
  interval: number;
}

export interface AuthenticatedUser {
  login: string;
  id: number;
  name: string | null;
  email: string | null;
  avatar_url: string;
  authenticated_at: string;
}

// AuthState matches Rust's serde(tag = "type") serialization
// The variant's fields are flattened into the same object
export type AuthState =
  | { type: "unauthenticated" }
  | ({ type: "pending" } & DeviceFlowPending)
  | ({ type: "authenticated" } & AuthenticatedUser);

// Repository types
export interface Repository {
  path: string;
  owner: string;
  name: string;
  current_branch: string;
  default_branch: string;
  remote_url: string;
  sync_state: SyncState;
}

export type SyncState = 
  | { type: "up_to_date" }
  | { type: "ahead"; commits: number }
  | { type: "behind"; commits: number }
  | { type: "diverged"; ahead: number; behind: number }
  | { type: "unknown" };
