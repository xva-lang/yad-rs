use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Comment {
    pub author_association: String,
    pub body: String,
    pub created_at: String,
    pub html_url: String,
    pub id: usize,
    pub issue_url: String,
    pub node_id: String,
    // url:
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct User {
    pub login: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PullRequest {
    pub url: String,
    pub html_url: String,
    pub diff_url: String,
    pub patch_url: String,
    pub merged_at: Option<String>,
}
