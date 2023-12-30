use crate::{
    config::{get_config, load_config},
    github::{create_issue_comment, GithubClient},
    logging::{error, info},
    routes::IssueCommentPayload,
};

pub(crate) async fn ping(ic: &IssueCommentPayload) {
    let owner = &ic.repository.owner.as_ref().unwrap().login;
    let repo = &ic.repository.name;
    let commenter = &ic.issue.user.login;

    info(
        format!("@{commenter} has tried to check whether service is still alive"),
        Some(&load_config(None).unwrap()),
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
    let config = get_config();

    // If the value is none, the commenter has issued the "claim" command - i.e. they are assigning themselves.
    let assignee = match &assignee {
        Some(v) => v.as_str(),
        None => commenter.as_str(),
    };

    // Assign the specified user and report the action as an issue comment
    let client = GithubClient::new(config.access_token());
    if let Err(e) = client
        .add_assignee_to_issue(owner, repo, issue_number, assignee)
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
    let config = get_config();

    // Check that the user that issued the command is one of the assignees already
    // If a user that issued this command is not already an assignee then no-op
    let client = GithubClient::new(config.access_token());

    let mut should_delete_assignee = false;
    match client.list_issue_assignees(owner, repo, issue_number).await {
        Ok(assignees) => {
            if assignees.iter().map(|x| &x.login).any(|x| x == commenter) {
                should_delete_assignee = true;
            }
        }
        Err(e) => error(
            format!("Failed to retrieve issue. Extended error: {e}"),
            Some(&config),
        ),
    }

    if should_delete_assignee {
        if let Err(e) = client
            .delete_assignee(owner, repo, issue_number, &commenter)
            .await
        {
            error(
                format!("Failed to delete assignee on issue #{issue_number}. Extended error: {e}"),
                Some(&config),
            );
        }
    }
}

// pub(crate) async fn approve_pull(ic: &IssueCommentPayload) {}

// pub(crate) async fn delete_assignee(
//     owner: &str,
//     repo: &str,
//     issue_number: u64,
//     assignee: &str,
// ) -> Result<(), reqwest::Error> {
//     //  -H "Accept: application/vnd.github+json" \
//     //   -H "Authorization: Bearer <YOUR-TOKEN>" \
//     //   -H "X-GitHub-Api-Version: 2022-11-28" \

//     let route =
//         format!("https://api.github.com/repos/{owner}/{repo}/issues/{issue_number}/assignees");
//     let config = load_config(None).unwrap();

//     #[derive(Serialize)]
//     struct DeleteAssignees<'a> {
//         // Zero allocations, in a serde Serializable? got me feelin some type of way!
//         assignees: &'a [&'a str],
//     }

//     let body = serde_json::to_string(&DeleteAssignees {
//         assignees: &[assignee],
//     })
//     .unwrap();

//     let client = reqwest::Client::new();
//     match client
//         .delete(route)
//         .bearer_auth(config.access_token())
//         .header("User-Agent", "yad")
//         .header("Accept", "application/vnd.github+json")
//         .header("X-GitHub-Api-Version", "2022-11-28")
//         .body(body)
//         .send()
//         .await
//     {
//         Ok(_) => Ok(()),
//         Err(e) => {
//             error(
//                 format!("Failed to delete assignee. Extended error: {e}"),
//                 Some(&config),
//             );
//             Err(e)
//         }
//     }
// }
