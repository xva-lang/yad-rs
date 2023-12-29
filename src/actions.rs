use octocrab::Octocrab;
use std::{error::Error, sync::Arc};
use url::Url;

use crate::{
    config::get_config,
    github::{create_issue_comment, delete_assignee},
    logging::{error, info},
    routes::IssueCommentPayload,
};

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

pub(crate) async fn ping(ic: &IssueCommentPayload) {
    let owner = &ic.repository.owner.as_ref().unwrap().login;
    let repo = &ic.repository.name;
    let commenter = &ic.issue.user.login;

    info(
        format!("@{commenter} has tried to check whether service is still alive"),
        Some(&get_config(None).unwrap()),
    );

    match create_issue_comment(
        owner,
        repo,
        ic.issue.number,
        &format!("Hi @{commenter}! Yes, I'm still alive!"),
    )
    .await
    {
        Ok(_) => {}
        Err(e) => panic!("{e}"),
    }
}

pub(crate) async fn set_assignee(ic: &IssueCommentPayload, assignee: Option<String>) {
    let owner = &ic.repository.owner.as_ref().unwrap().login;
    let repo = &ic.repository.name;
    let commenter = &ic.issue.user.login;
    let issue_number = ic.issue.number;
    let config = get_config(None).unwrap();

    // If the value is none, the commenter has issued the "claim" command - i.e. they are assigning themselves.
    let assignee = match &assignee {
        Some(v) => [v.as_str()],
        None => [commenter.as_str()],
    };

    // Assign the specified user and report the action as an issue comment
    if let Err(e) = octocrab::instance()
        .issues(owner, repo)
        .add_assignees(issue_number, &assignee)
        .await
    {
        error(
            format!("Failed to add assignee to issue #{issue_number}. Extended error: {e}"),
            Some(&config),
        )
    }
}

pub(crate) async fn remove_assignee(ic: &IssueCommentPayload) {
    let owner = &ic.repository.owner.as_ref().unwrap().login;
    let repo = &ic.repository.name;
    let commenter = &ic.issue.user.login;
    let issue_number = ic.issue.number;
    let config = get_config(None).unwrap();

    // Check that the user that issued the command is one of the assignees already
    // If a user that issued this command is not already an assignee then no-op
    let client = octocrab::instance();
    match client.issues(owner, repo).get(issue_number).await {
        Ok(i) => {
            if i.assignees.iter().map(|x| &x.login).any(|x| x == commenter) {
                if let Err(e) = delete_assignee(owner, repo, issue_number, commenter).await {
                    error(
                        format!("Failed to remove assignee @{commenter}. Extended error: {e}"),
                        Some(&config),
                    )
                }
            }
        }
        Err(e) => error(
            format!("Failed to retrieve issue. Extended error: {e}"),
            Some(&config),
        ),
    }
}

// pub(crate) async fn approve_pull(ic: &IssueCommentPayload) {}
