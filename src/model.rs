use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) enum PullRequestStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PullRequest {
    id: u64,
    repository: String,
    status: PullRequestStatus,
    merge_commit_id: String,
    head_commit_id: String,
    head_ref: String,
    base_ref: String,
    assignee: String,
    approved_by: String,
    priority: u64,
    try_test: bool,
    rollup: bool,
    squash: bool,
    delegate: String,
}
