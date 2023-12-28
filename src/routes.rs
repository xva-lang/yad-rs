use axum::{debug_handler, extract::State, http::HeaderMap, Json};
use octocrab::models::{
    events::payload::IssueCommentEventAction,
    issues::{Comment, Issue},
};
use serde::Deserialize;

use crate::{
    command::{parse_command, Command},
    AppState,
};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum EventPayload {
    IssueComment(IssueCommentPayload),
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub(crate) struct IssueCommentPayload {
    pub action: IssueCommentEventAction,
    pub issue: Issue,
    pub comment: Comment,

    pub repository: octocrab::models::Repository,
}

#[debug_handler]
pub(crate) async fn post_github(
    _headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<EventPayload>,
) {
    match payload {
        EventPayload::IssueComment(ic) => {
            if let Some(comment_body) = &ic.comment.body {
                let commands = parse_command(&state.app_user.login, &comment_body);

                for command in commands {
                    match command {
                        Command::Approve => crate::actions::approve_pull(&ic).await,
                        // Command::Assign { users } => crate::actions::assign_users(&ic).await,
                        _ => {}
                    }
                }
            }
        }
    }
}
