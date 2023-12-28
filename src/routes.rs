use axum::{http::HeaderMap, Json};
use octocrab::models::{
    events::payload::IssueCommentEventAction,
    issues::{Comment, Issue},
    pulls::PullRequest,
    Repository,
};
use serde::Deserialize;
use serde_json::Value;
use std::{fs::OpenOptions, io::Write};

use octocrab::models::events::payload::IssueCommentEventPayload;

use crate::github::create_issue_comment;

// fn execute_action(github_event: &str, payload: EventPayload) {
//     match github_event {
//         "issue_comment" => handlers::handle_issue_comment()
//         _ => panic!("Unknown GitHub event {github_event}")
//     }
// }

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum EventPayload {
    IssueComment(IssueCommentPayload),
}

#[derive(Debug, Deserialize)]
pub(crate) struct IssueCommentPayload {
    pub action: IssueCommentEventAction,
    pub issue: Issue,
    pub comment: Comment,

    pub repository: octocrab::models::Repository,
}

pub(crate) async fn post_github(headers: HeaderMap, Json(payload): Json<EventPayload>) {
    match payload {
        EventPayload::IssueComment(ic) => {
            let owner = &ic.repository.owner.unwrap().login;
            let repo = &ic.repository.name;
            let commenter = &ic.issue.user.login;

            let client = octocrab::instance();
            let pr_url = match client.issues(owner, repo).get(ic.issue.number).await {
                Ok(iss) => {
                    println!("Issue: {iss:#?}");
                    if let Some(pr) = iss.pull_request {
                        Some(pr.url)
                    } else {
                        None
                    }
                }
                Err(e) => panic!("Error getting issue: {e}"),
            };

            let commit_id = if let Some(pr_url) = pr_url {
                println!("pr_url: {pr_url}");

                let pr: PullRequest = client
                    .pulls(owner, repo)
                    .get(
                        pr_url
                            .path_segments()
                            .unwrap()
                            .last()
                            .unwrap()
                            .parse()
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                println!("{pr:#?}");

                pr.merge_commit_sha
            } else {
                None
            };

            let body = format!(
                "📌 {} has been approved by @{commenter}\n\nIt is now in the queue for this repository.",
                if let Some(ci) = commit_id {
                    format!("Commit {ci} ")
                } else {
                    "Pull request ".into()
                }
            );

            match create_issue_comment(owner, repo, ic.issue.number, &body).await {
                Ok(_) => {}
                Err(e) => panic!("{e}"),
            }
        }
    }
}
