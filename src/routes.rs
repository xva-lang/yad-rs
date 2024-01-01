use axum::{body::Body, debug_handler, extract::State, http::HeaderMap, Json};

use serde::Deserialize;
use url::Url;

use crate::{
    actions::{
        approve_pull, ping, remove_assignee, save_pull_to_db, set_assignee,
        set_pull_request_approved, set_pull_request_status,
    },
    command::{parse_command, Command},
    config::get_config,
    github::model::{
        checks::CheckSuite,
        pulls::{PullRequest, PullRequestReview, PullRequestReviewState},
        repo::Repository,
        Comment, Issue, IssueCommentEventAction,
    },
    logging::error,
    model::PullRequestStatus,
    AppState,
};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum EventPayload {
    IssueComment(IssueCommentPayload),
    PullRequest(PullRequestPayload),
    // CheckSuite(CheckSuitePayload), // PullRequestReview(PullRequestReviewPayload),
    CheckRun(CheckRunPayload),
}

#[derive(Debug, Deserialize)]

pub(crate) enum PullRequestEventAction {
    #[serde(rename = "opened")]
    Opened,

    #[serde(rename = "closed")]
    Closed,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub(crate) struct PullRequestPayload {
    action: PullRequestEventAction,
    number: u64,
    pull_request: PullRequest,
    repository: Repository,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub(crate) struct IssueCommentPayload {
    pub action: IssueCommentEventAction,
    pub issue: Issue,
    pub comment: Comment,

    pub repository: Repository,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum PullRequestReviewEventAction {
    Dismissed,
    Edited,
    Submitted,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub(crate) struct PullRequestReviewPayload {
    pub action: PullRequestReviewEventAction,
    pub review: PullRequestReview,
    pub pull_request: PullRequest,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CheckSuiteEventAction {
    Completed,
    Requested,
    Rerequested,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CheckSuitePayload {
    pub action: CheckSuiteEventAction,
    pub check_suite: CheckSuite,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CheckRunEventAction {
    Created,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CheckRunPayload {
    action: CheckRunEventAction,
    id: u64,
    name: String,
    head_sha: String,
    url: Url,
    check_suite: CheckSuite,
}

const GITHUB_EVENT_KEY: &str = "X-GitHub-Event";
const GITHUB_EVENT_ISSUE_COMMENT: &str = "issue_comment";
const GITHUB_EVENT_PULL_REQUEST: &str = "pull_request";
const GITHUB_EVENT_PULL_REQUEST_REVIEW: &str = "pull_request_review";
const GITHUB_EVENT_CHECK_SUITE: &str = "check_suite";
const GITHUB_EVENT_CHECK_RUN: &str = "check_run";

#[debug_handler]
pub(crate) async fn post_github(
    headers: HeaderMap,
    State(state): State<AppState>,
    body: String, // Json(payload): Json<EventPayload>,
) {
    let config = get_config();
    let event_type = match headers.get(GITHUB_EVENT_KEY) {
        Some(et) => et,
        None => {
            error("No X-GitHub-Event key provided.".into(), Some(&config));
            return;
        }
    };

    let payload = match event_type.to_str().unwrap() {
        GITHUB_EVENT_ISSUE_COMMENT => {
            EventPayload::IssueComment(serde_json::from_str::<IssueCommentPayload>(&body).unwrap())
        }
        GITHUB_EVENT_PULL_REQUEST => {
            EventPayload::PullRequest(serde_json::from_str::<PullRequestPayload>(&body).unwrap())
        }
        GITHUB_EVENT_CHECK_RUN => EventPayload::CheckRun(serde_json::from_str(&body).unwrap()),
        // GITHUB_EVENT_PULL_REQUEST_REVIEW => EventPayload::PullRequestReview(
        //     serde_json::from_str::<PullRequestReviewPayload>(&body).unwrap(),
        // ),
        _ => {
            error(format!("Unknown event {event_type:#?}"), Some(&config));
            return;
        }
    };

    match payload {
        EventPayload::IssueComment(ic) => {
            if let Some(comment_body) = &ic.comment.body {
                let commands = parse_command(&state.app_user.login, &comment_body);

                for command in commands {
                    match command {
                        Command::Approve => approve_pull(&ic).await,
                        Command::Ping => ping(&ic).await,
                        Command::Assign { user } => set_assignee(&ic, user).await,
                        Command::RemoveAssignment => remove_assignee(&ic).await,
                        _ => {}
                    }
                }
            }
        }
        EventPayload::PullRequest(PullRequestPayload {
            action,
            number,
            pull_request,
            repository,
        }) => match action {
            PullRequestEventAction::Opened => {
                save_pull_to_db(pull_request, repository).await.unwrap()
            }
            PullRequestEventAction::Closed => {
                if let Some(_) = pull_request.merged_at {
                    set_pull_request_status(pull_request.id, PullRequestStatus::Merged)
                        .await
                        .unwrap()
                } else {
                    set_pull_request_status(pull_request.id, PullRequestStatus::Closed)
                        .await
                        .unwrap()
                }
            }
        },

        EventPayload::CheckRun(CheckRunPayload {
            action,
            id,
            name,
            head_sha,
            url,
            check_suite,
        }) => match action {
            CheckRunEventAction::Created => {}
        },
    }
}
