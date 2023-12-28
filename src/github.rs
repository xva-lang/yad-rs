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
