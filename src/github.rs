pub(crate) mod model;

use crate::config::load_config;
use lazy_static::lazy_static;
use model::User;
use reqwest::{IntoUrl, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug)]
pub(crate) enum GithubClientError {
    GithubError(Response),
    RequestError(reqwest::Error),
}

impl std::fmt::Display for GithubClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for GithubClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

lazy_static! {
    static ref REQWEST_CLIENT: reqwest::Client = reqwest::Client::builder()
        .user_agent("yad")
        .build()
        .unwrap();
}

const GITHUB_API_ROOT: &str = "https://api.github.com";
const GITHUB_ACCEPT_TYPE: &str = "application/vnd.github+json";
const GITHUB_API_VERSION_HEADER_KEY: &str = "X-GitHub-Api-Version";
const GITHUB_API_VERSION_HEADER_VALUE: &str = "2022-11-28";

pub(crate) struct GithubClient<'a> {
    reqwest: reqwest::Client,
    access_token: &'a str, // default_headers: &'a [(&'a str, &'a str)],
}

impl<'a> GithubClient<'a> {
    pub(crate) fn new(access_token: &'a str) -> Self {
        let reqwest_client = REQWEST_CLIENT.clone();

        Self {
            reqwest: reqwest_client,
            access_token,
        }
    }

    async fn get<U: IntoUrl>(&self, route: U) -> Result<reqwest::Response, reqwest::Error> {
        let bearer = format!("Bearer {}", self.access_token);
        let default_headers = &[
            ("Authorization", bearer.as_str()),
            ("Accept", GITHUB_ACCEPT_TYPE),
            (
                GITHUB_API_VERSION_HEADER_KEY,
                GITHUB_API_VERSION_HEADER_VALUE,
            ),
        ];

        let mut builder = self.reqwest.get(route);
        for (k, v) in default_headers {
            builder = builder.header(*k, *v)
        }

        let request = builder.build().unwrap();
        self.reqwest.execute(request).await
    }

    async fn post<U, T>(&self, route: U, body: &T) -> Result<reqwest::Response, reqwest::Error>
    where
        U: IntoUrl,
        T: Serialize + ?Sized,
    {
        let bearer = format!("Bearer {}", self.access_token);
        let default_headers = &[
            ("Authorization", bearer.as_str()),
            ("Accept", GITHUB_ACCEPT_TYPE),
            (
                GITHUB_API_VERSION_HEADER_KEY,
                GITHUB_API_VERSION_HEADER_VALUE,
            ),
        ];

        let mut builder = self.reqwest.post(route);
        for (k, v) in default_headers {
            builder = builder.header(*k, *v)
        }

        let request = builder
            .body(serde_json::to_string(body).unwrap())
            .build()
            .unwrap();
        self.reqwest.execute(request).await
    }

    async fn delete<U, T>(
        &self,
        route: U,
        body: Option<&T>,
    ) -> Result<reqwest::Response, reqwest::Error>
    where
        U: IntoUrl,
        T: Serialize + ?Sized,
    {
        let bearer = format!("Bearer {}", self.access_token);
        let default_headers = &[
            ("Authorization", bearer.as_str()),
            ("Accept", GITHUB_ACCEPT_TYPE),
            (
                GITHUB_API_VERSION_HEADER_KEY,
                GITHUB_API_VERSION_HEADER_VALUE,
            ),
        ];

        let mut builder = self.reqwest.delete(route);
        for (k, v) in default_headers {
            builder = builder.header(*k, *v)
        }

        if let Some(b) = body {
            builder = builder.body(serde_json::to_string(b).unwrap())
        }

        let request = builder.build().unwrap();
        self.reqwest.execute(request).await
    }

    pub(crate) async fn create_issue_comment(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        body: &str,
    ) -> Result<(), GithubClientError> {
        let route =
            format!("{GITHUB_API_ROOT}/repos/{owner}/{repo}/issues/{issue_number}/comments");

        #[derive(Serialize)]
        struct PostIssueComment<'a> {
            body: &'a str,
        }

        match self.post(route, &PostIssueComment { body }).await {
            Ok(r) => match r.status() {
                StatusCode::CREATED => Ok(()),
                _ => Err(GithubClientError::GithubError(r)),
            },
            Err(e) => Err(GithubClientError::RequestError(e)),
        }
    }

    pub(crate) async fn list_issue_assignees(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<Vec<User>, Box<dyn Error>> {
        //https://api.github.com/repos/{{owner}}/{{repo}}/issues/{{issue_number}}
        let route = format!("{GITHUB_API_ROOT}/repos/{owner}/{repo}/issues/{issue_number}");

        #[derive(Debug, Deserialize)]
        struct ListAssigneesResponse {
            assignees: Vec<User>,
        }

        let response = match self.get(route).await {
            Ok(r) => r,
            Err(e) => return Err(Box::from(e)),
        };

        let resp_text = response.text().await.unwrap();
        let resp = serde_json::from_str::<ListAssigneesResponse>(&resp_text).unwrap();

        Ok(resp.assignees)
    }

    pub(crate) async fn add_assignee_to_issue(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        assignee: &str,
    ) -> Result<(), GithubClientError> {
        let route =
            format!("{GITHUB_API_ROOT}/repos/{owner}/{repo}/issues/{issue_number}/assignees");
        #[derive(Serialize)]
        struct PostAssignees<'a> {
            assignees: &'a [&'a str],
        }

        match self
            .post(
                route,
                &PostAssignees {
                    assignees: &[assignee],
                },
            )
            .await
        {
            Ok(r) => match r.status() {
                StatusCode::CREATED => Ok(()),
                _ => Err(GithubClientError::GithubError(r)),
            },
            Err(e) => Err(GithubClientError::RequestError(e)),
        }
    }

    pub(crate) async fn delete_assignee(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        assignee: &str,
    ) -> Result<(), GithubClientError> {
        let route =
            format!("{GITHUB_API_ROOT}/repos/{owner}/{repo}/issues/{issue_number}/assignees");
        #[derive(Serialize)]
        struct DeleteAssignees<'a> {
            assignees: &'a [&'a str],
        }

        match self
            .delete(
                route,
                Some(&DeleteAssignees {
                    assignees: &[assignee],
                }),
            )
            .await
        {
            Ok(r) => match r.status() {
                StatusCode::OK => Ok(()),
                _ => Err(GithubClientError::GithubError(r)),
            },
            Err(e) => Err(GithubClientError::RequestError(e)),
        }
    }

    pub(crate) async fn get_authenticated_user(&self) -> Result<User, GithubClientError> {
        let route = format!("{GITHUB_API_ROOT}/user");

        let response = match self.get(route).await {
            Ok(r) => match r.status() {
                StatusCode::OK | StatusCode::NOT_MODIFIED => r,
                _ => return Err(GithubClientError::GithubError(r)),
            },
            Err(e) => return Err(GithubClientError::RequestError(e)),
        };

        Ok(serde_json::from_str::<User>(&response.text().await.unwrap()).unwrap())
    }
}

pub(crate) async fn create_issue_comment(
    owner: &str,
    repo: &str,
    issue_number: u64,
    body: &str,
) -> Result<(), Box<dyn Error>> {
    let config = load_config(None).unwrap();
    let client = GithubClient::new(config.access_token());
    match client
        .create_issue_comment(owner, repo, issue_number, body)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::from(e)),
    }
}

#[cfg(test)]
mod tests {
    use crate::config::get_config;

    use super::GithubClient;

    #[tokio::test]
    async fn test_create_issue_comment() {
        let config = get_config();
        let client = GithubClient::new(config.access_token());
        client
            .create_issue_comment("xva-lang", "homu-test-repo", 4, "test comment")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_delete_assignee() {
        let config = get_config();
        let client = GithubClient::new(&config.access_token());
        client
            .delete_assignee("xva-lang", "homu-test-repo", 4, "dylangiles")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_add_assignee_to_issue() {
        let config = get_config();
        let client = GithubClient::new(&config.access_token());
        client
            .add_assignee_to_issue("xva-lang", "homu-test-repo", 4, "dylangiles")
            .await
            .unwrap();
    }
}
