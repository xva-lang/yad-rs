use axum::http::request;
use serde::{Deserialize, Serialize};

use crate::{config::get_config, logging::error};

pub(crate) mod model;

pub(crate) async fn create_issue_comment(
    owner: &str,
    repo: &str,
    issue_number: u64,
    body: &str,
) -> Result<(), octocrab::Error> {
    let client = octocrab::instance();
    let _ = client
        .issues(owner, repo)
        .create_comment(issue_number, body)
        .await?;

    Ok(())
}

pub(crate) async fn delete_assignee(
    owner: &str,
    repo: &str,
    issue_number: u64,
    assignee: &str,
) -> Result<(), reqwest::Error> {
    //  -H "Accept: application/vnd.github+json" \
    //   -H "Authorization: Bearer <YOUR-TOKEN>" \
    //   -H "X-GitHub-Api-Version: 2022-11-28" \

    let route =
        format!("https://api.github.com/repos/{owner}/{repo}/issues/{issue_number}/assignees");
    let config = get_config(None).unwrap();

    #[derive(Serialize)]
    struct DeleteAssignees<'a> {
        // Zero allocations, in a serde Serializable? got me feelin some type of way!
        assignees: &'a [&'a str],
    }

    let body = serde_json::to_string(&DeleteAssignees {
        assignees: &[assignee],
    })
    .unwrap();

    let client = reqwest::Client::new();
    match client
        .delete(route)
        .bearer_auth(config.access_token())
        .header("User-Agent", "yad")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .body(body)
        .send()
        .await
    {
        Ok(r) => {
            println!("{}", r.text().await.unwrap());
            Ok(())
        }
        Err(e) => {
            error(
                format!("Failed to delete assignee. Extended error: {e}"),
                Some(&config),
            );
            Err(e)
        }
    }
}
