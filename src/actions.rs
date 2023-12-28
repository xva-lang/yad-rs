use octocrab::Octocrab;
use std::{error::Error, sync::Arc};
use url::Url;

use crate::{github::create_issue_comment, routes::IssueCommentPayload};

async fn get_pr_url(
    owner: &str,
    repo: &str,
    issue_number: u64,
) -> Result<Option<Url>, Box<dyn Error>> {
    let client = octocrab::instance();
    match (client.issues(owner, repo).get(issue_number).await?).pull_request {
        Some(pr) => Ok(Some(pr.url)),
        None => Ok(None),
    }
}

async fn get_commit_id_from_pull_request(
    owner: &str,
    repo: &str,
    url: Url,
    gh_instance: Option<Arc<Octocrab>>,
) -> Option<String> {
    let client = match gh_instance {
        Some(i) => i,
        None => octocrab::instance(),
    };

    match client
        .pulls(owner, repo)
        .get(
            url.path_segments()
                .unwrap()
                .last()
                .unwrap()
                .parse()
                .unwrap(),
        )
        .await
    {
        Ok(pr) => pr.merge_commit_sha.map_or(None, |sha| Some(sha)),
        Err(e) => {
            println!("Error getting pull request: {e}");
            None
        }
    }
}

pub(crate) async fn approve_pull(ic: &IssueCommentPayload) {
    let owner = &ic.repository.owner.as_ref().unwrap().login;
    let repo = &ic.repository.name;
    let commenter = &ic.issue.user.login;
    let issue_number = ic.issue.number;

    let client = octocrab::instance();
    let pr_url = match get_pr_url(owner, repo, issue_number).await {
        Ok(url) => url,
        Err(e) => {
            println!("Error getting url: {e}");
            None
        }
    };

    let commit_id = if let Some(url) = pr_url {
        get_commit_id_from_pull_request(owner, repo, url, Some(client)).await
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

    match create_issue_comment(&owner, repo, ic.issue.number, &body).await {
        Ok(_) => {}
        Err(e) => panic!("{e}"),
    }
}

// pub(crate) async fn approve_pull(ic: &IssueCommentPayload) {}
