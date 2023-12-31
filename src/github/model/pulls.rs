use chrono::{DateTime, Local};
use serde::Deserialize;

use super::{repo::Repository, User};

#[derive(Debug, Deserialize)]
pub(crate) struct PullRequest {
    pub id: u64,
    // pub repository: Repository,
    pub number: u64,
    pub head: Head,
    pub base: Base,
    pub merge_commit_sha: Option<String>,
    pub assignee: Option<User>,
    pub merged_at: Option<DateTime<Local>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Head {
    pub label: Option<String>,
    #[serde(rename = "ref")]
    pub ref_field: String,
    pub sha: String,
    pub user: Option<User>,
    pub repo: Option<Repository>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Base {
    pub label: Option<String>,
    #[serde(rename = "ref")]
    pub ref_field: String,
    pub sha: String,
    pub user: Option<User>,
    pub repo: Option<Repository>,
}

#[derive(Debug, Deserialize)]
pub(crate) enum PullRequestReviewState {
    /// A review allowing the pull request to merge.
    #[serde(alias = "APPROVED")]
    #[serde(alias = "approved")]
    Approved,

    /// A review blocking the pull request from merging.
    #[serde(alias = "CHANGES_REQUESTED")]
    #[serde(alias = "changes_requested")]
    ChangesRequested,

    /// An informational review.
    #[serde(alias = "COMMENTED")]
    #[serde(alias = "commented")]
    Commented,

    /// A review that has been dismissed.
    #[serde(alias = "DISMISSED")]
    #[serde(alias = "dismissed")]
    Dismissed,

    #[serde(alias = "PENDING")]
    #[serde(alias = "pending")]
    Pending,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PullRequestReview {
    id: u64,
    pub state: PullRequestReviewState,
    pub user: User,
}
