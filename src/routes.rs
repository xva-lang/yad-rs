use axum::{body::Body, debug_handler, extract::State, http::HeaderMap, Json};

use serde::Deserialize;

use crate::{
    actions::{ping, remove_assignee, save_pull_to_db, set_assignee},
    command::{parse_command, Command},
    config::get_config,
    github::model::{
        pulls::PullRequest, repo::Repository, Comment, Issue, IssueCommentEventAction,
    },
    logging::error,
    AppState,
};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum EventPayload {
    IssueComment(IssueCommentPayload),
    PullRequest(PullRequestPayload),
}

#[derive(Debug, Deserialize)]

pub(crate) enum PullRequestEventAction {
    #[serde(rename = "opened")]
    Opened,
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

const GITHUB_EVENT_KEY: &str = "X-GitHub-Event";
const GITHUB_EVENT_ISSUE_COMMENT: &str = "issue_comment";
const GITHUB_EVENT_PULL_REQUEST: &str = "pull_request";

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
                        // Command::Approve => crate::actions::approve_pull(&ic).await,
                        Command::Ping => ping(&ic).await,
                        Command::Assign { user } => set_assignee(&ic, user).await,
                        Command::RemoveAssignment => remove_assignee(&ic).await,
                        _ => {}
                    }
                }
            }
        }
        EventPayload::PullRequest(p) => {
            save_pull_to_db(p.pull_request, p.repository).await.unwrap()
        }
    }
}
