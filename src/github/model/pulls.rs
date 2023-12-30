use serde::Deserialize;

use super::{repo::Repository, User};

#[derive(Debug, Deserialize)]
pub(crate) struct PullRequest {
    pub id: u64,
    // pub repository: Repository,
    pub head: Head,
    pub base: Base,
    pub merge_commit_sha: Option<String>,
    pub assignee: Option<User>,
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
