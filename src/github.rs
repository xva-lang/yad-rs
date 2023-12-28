use axum::http::request;

use crate::config::get_config;

pub(crate) mod model;

pub(crate) async fn create_issue_comment(
    owner: &str,
    repo: &str,
    issue_number: u64,
    body: &str,
) -> Result<(), octocrab::Error> {
    //repos/{owner}/{repo}/pulls/{pull_number}/comments
    let config = get_config(None).unwrap();
    let repo_config = config.get_repo(repo).unwrap();

    let url = format!("https://api.github.com/repos/{owner}/{repo}/issues/{issue_number}/comments");
    let auth = format!("Bearer {}", config.access_token());

    octocrab::initialise(
        octocrab::Octocrab::builder()
            .personal_token(config.access_token().to_string())
            .build()
            .unwrap(),
    );
    let client = octocrab::instance();
    let resp = client
        .issues(owner, repo)
        .create_comment(issue_number, body)
        .await?;

    Ok(())
}
