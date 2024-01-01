use serde::Deserialize;
use url::Url;

use super::pulls::PullRequest;

#[derive(Debug, Deserialize)]
pub(crate) enum CheckSuiteStatus {
    // requested, in_progress, completed, queued, null, pending
    #[serde(rename = "requested")]
    Requested,

    #[serde(rename = "in_progress")]
    InProgress,

    #[serde(rename = "completed")]
    Completed,

    #[serde(rename = "queued")]
    Queued,

    #[serde(rename = "pending")]
    Pending,
}

#[derive(Debug, Deserialize)]
pub(crate) enum CheckSuiteConclusion {
    #[serde(rename = "success")]
    Success,

    #[serde(rename = "failure")]
    Failure,

    #[serde(rename = "neutral")]
    Neutral,

    #[serde(rename = "cancelled")]
    Cancelled,

    #[serde(rename = "timed_out")]
    TimedOut,

    #[serde(rename = "action_required")]
    ActionRequired,

    #[serde(rename = "stale")]
    Stale,

    #[serde(rename = "skipped")]
    Skipped,

    #[serde(rename = "startup_failure")]
    StartupFailure,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CheckSuite {
    id: u64,
    head_branch: String,
    head_sha: String,
    status: Option<CheckSuiteStatus>,
    conclusion: Option<CheckSuiteConclusion>,
    url: Url,
    before: String,
    after: String,
    pull_requests: Vec<CheckSuitePullRequest>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CheckSuitePullRequestHead {
    ref_name: String,
    sha: String,
    repo: CheckSuitePullRequestRepo,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CheckSuitePullRequestBase {
    ref_name: String,
    sha: String,
    repo: CheckSuitePullRequestRepo,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CheckSuitePullRequestRepo {
    id: u64,
    url: Url,
    name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CheckSuitePullRequest {
    url: Url,
    id: u64,
    number: u64,
    head: CheckSuitePullRequestHead,
    base: CheckSuitePullRequestBase,
}
