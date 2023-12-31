pub(crate) mod model;

use crate::{
    config::{get_config, load_config},
    github::model::pulls::PullRequestReviewState,
};
use chrono::{DateTime, Duration, Local};
use lazy_static::lazy_static;
use model::User;
use reqwest::{IntoUrl, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, error::Error};

use self::model::{pulls::PullRequest, Issue};

#[derive(Debug)]
pub(crate) enum GithubClientError {
    GithubError(Response),
    RequestError(reqwest::Error),
    Basic(String),
}

const CHECK_FORBIDDEN_ERROR: &str =
    "Forbidden. The provided token likely does not have permission to create checks.";

impl std::fmt::Display for GithubClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GithubClientError::GithubError(g) => write!(f, "Github error: {g:#?}"),
            GithubClientError::RequestError(r) => write!(f, "Net error: {r}"),
            GithubClientError::Basic(s) => write!(f, "{s}"),
        }
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

pub(crate) const GITHUB_API_ROOT: &str = "https://api.github.com";
const GITHUB_ACCEPT_TYPE: &str = "application/vnd.github+json";
const GITHUB_API_VERSION_HEADER_KEY: &str = "X-GitHub-Api-Version";
const GITHUB_API_VERSION_HEADER_VALUE: &str = "2022-11-28";

pub(crate) struct GithubClient<'a> {
    reqwest: reqwest::Client,
    access_token: &'a str, // default_headers: &'a [(&'a str, &'a str)],
    gh_app_token: String,
}

impl<'a> GithubClient<'a> {
    pub(crate) fn new(access_token: &'a str) -> Self {
        let reqwest_client = REQWEST_CLIENT.clone();

        Self {
            reqwest: reqwest_client,
            access_token,
            gh_app_token: "".into(),
        }
    }

    async fn get<U: IntoUrl>(
        &self,
        route: U,
        bearer_override: Option<&str>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let bearer = format!(
            "Bearer {}",
            if let Some(bo) = bearer_override {
                bo
            } else {
                self.access_token
            }
        );
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

    async fn post<U, T>(
        &self,
        route: U,
        body: Option<&T>,
        bearer_override: Option<&str>,
    ) -> Result<reqwest::Response, reqwest::Error>
    where
        U: IntoUrl,
        T: Serialize + ?Sized,
    {
        let bearer = format!(
            "Bearer {}",
            if let Some(bo) = bearer_override {
                bo
            } else {
                self.access_token
            }
        );
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

        let request = if let Some(b) = body {
            builder
                .body(serde_json::to_string(b).unwrap())
                .build()
                .unwrap()
        } else {
            builder.build().unwrap()
        };
        self.reqwest.execute(request).await
    }

    async fn gh_app_post<U, T>(
        &mut self,
        route: U,
        body: Option<&T>,
        owner: &str,
        repo: &str,
    ) -> Result<reqwest::Response, GithubClientError>
    where
        U: IntoUrl,
        T: Serialize + ?Sized,
    {
        if self.gh_app_token == "" {
            self.authorise_gh_app(owner, repo).await.unwrap();
        }

        let inner = || async {
            let bearer = format!("Bearer {}", &self.gh_app_token);

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

            let request = if let Some(b) = body {
                builder
                    .body(serde_json::to_string(b).unwrap())
                    .build()
                    .unwrap()
            } else {
                builder.build().unwrap()
            };

            let cloned_request = request.try_clone().unwrap();
            match self.reqwest.execute(request).await {
                Ok(r) => match r.status() {
                    StatusCode::FORBIDDEN => {
                        if let Err(e) = self.authorise_gh_app(owner, repo).await {
                            return Err(e);
                        }
                        match self.reqwest.execute(cloned_request).await {
                            Ok(r) => Ok(r),
                            Err(e) => return Err(GithubClientError::RequestError(e)),
                        }
                    }
                    _ => Ok(r),
                },
                Err(e) => return Err(GithubClientError::RequestError(e)),
            }
        };

        inner().await
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

        match self
            .post(route, Some(&PostIssueComment { body }), None)
            .await
        {
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

        let response = match self.get(route, None).await {
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
                Some(&PostAssignees {
                    assignees: &[assignee],
                }),
                None,
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

        let response = match self.get(route, None).await {
            Ok(r) => match r.status() {
                StatusCode::OK | StatusCode::NOT_MODIFIED => r,
                _ => return Err(GithubClientError::GithubError(r)),
            },
            Err(e) => return Err(GithubClientError::RequestError(e)),
        };

        Ok(serde_json::from_str::<User>(&response.text().await.unwrap()).unwrap())
    }

    fn generate_jwt(&self) -> String {
        let config = get_config();

        let private_pem = std::fs::read(&config.github.app.private_key_file).unwrap();

        let iat_claim = chrono::offset::Local::now().timestamp();
        let exp_claim = chrono::offset::Local::now()
            .checked_add_signed(Duration::minutes(10))
            .unwrap()
            .timestamp();

        use jsonwebtoken::{
            decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation,
        };
        /// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
        #[derive(Debug, Serialize)]
        struct Claims<'a> {
            iss: &'a str,
            exp: i64,
            iat: i64,
        }

        let claims = Claims {
            iss: &config.github.app.app_id,
            exp: exp_claim,
            iat: iat_claim,
        };

        encode(
            &Header::new(Algorithm::RS256),
            &claims,
            &EncodingKey::from_rsa_pem(&private_pem).unwrap(),
        )
        .unwrap()
    }
    pub(crate) async fn create_check(
        &mut self,
        owner: &str,
        repo: &str,
        head_sha: &str,
    ) -> Result<(), GithubClientError> {
        let route = format!("{GITHUB_API_ROOT}/repos/{owner}/{repo}/check-runs");

        #[derive(Serialize)]
        struct PostCheckRun<'a> {
            name: &'a str,
            head_sha: &'a str,
            status: &'a str,
            started_at: DateTime<Local>,
        }

        match self
            .gh_app_post(
                route,
                Some(&PostCheckRun {
                    name: "yad: check",
                    head_sha,
                    status: "queued",
                    started_at: chrono::offset::Local::now(),
                }),
                owner,
                repo,
            )
            .await
        {
            Ok(r) => match r.status() {
                StatusCode::CREATED => Ok(()),
                StatusCode::FORBIDDEN => {
                    return Err(GithubClientError::Basic(CHECK_FORBIDDEN_ERROR.into()))
                }
                _ => return Err(GithubClientError::GithubError(r)),
            },
            Err(e) => return Err(e),
        }
    }

    async fn authorise_gh_app(&mut self, owner: &str, repo: &str) -> Result<(), GithubClientError> {
        let route = format!("{GITHUB_API_ROOT}/repos/{owner}/{repo}/installation");
        #[derive(Deserialize)]
        struct InstallationResponse {
            id: u64,
        }

        let jwt = &self.generate_jwt();
        let response = match self.get(route, Some(jwt)).await {
            Ok(r) => r,
            Err(e) => return Err(GithubClientError::RequestError(e)),
        };

        let installation_id =
            serde_json::from_str::<InstallationResponse>(&response.text().await.unwrap())
                .unwrap()
                .id;

        #[derive(Deserialize)]
        struct AccessTokensResponse {
            token: String,
        }

        let route = format!("{GITHUB_API_ROOT}/app/installations/{installation_id}/access_tokens");
        let response = match self.post(route, <Option<&()>>::None, Some(jwt)).await {
            Ok(r) => r,
            Err(e) => return Err(GithubClientError::RequestError(e)),
        };

        println!("{response:#?}");
        self.gh_app_token =
            serde_json::from_str::<AccessTokensResponse>(&response.text().await.unwrap())
                .unwrap()
                .token;

        Ok(())
    }

    pub(crate) async fn get_pull_request_from_issue_number(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<Option<PullRequest>, GithubClientError> {
        let route = format!("{GITHUB_API_ROOT}/repos/{owner}/{repo}/issues/{issue_number}");
        let response = match self.get(route, None).await {
            Ok(r) => r,
            Err(e) => return Err(GithubClientError::RequestError(e)),
        };

        match serde_json::from_str::<Issue>(&response.text().await.unwrap())
            .unwrap()
            .pull_request
        {
            Some(pr_url) => match self.get(pr_url.url, None).await {
                Ok(r) => {
                    let pr = serde_json::from_str::<PullRequest>(&r.text().await.unwrap()).unwrap();
                    Ok(Some(pr))
                }
                Err(e) => return Err(GithubClientError::RequestError(e)),
            },
            None => Ok(None),
        }
    }

    pub(crate) async fn add_approved_review(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        commit_id: &str,
        on_behalf_of: Option<&str>,
    ) -> Result<(), GithubClientError> {
        let route = format!("{GITHUB_API_ROOT}/repos/{owner}/{repo}/pulls/{pull_number}/reviews");

        #[derive(Debug, Serialize)]
        struct PostReview<'a> {
            commit_id: &'a str,
            body: String,
            event: &'a str,
            comments: &'a [()],
        }

        let message = match on_behalf_of {
            Some(obo) => {
                format!(
                    r"
:pushpin: Commit {commit_id} has been approved by `{obo}`

It is now in the queue for this repository."
                )
            }
            None => {
                format!(
                    r"
:pushpin: Commit {commit_id} has been approved.

It is now in the queue for this repository"
                )
            }
        };

        let body = PostReview {
            commit_id,
            body: message,
            event: "APPROVE",
            comments: &[],
        };

        match self.post(route, Some(&body), None).await {
            Ok(r) => match r.status() {
                StatusCode::OK => Ok(()),
                _ => Err(GithubClientError::GithubError(r)),
            },
            Err(e) => Err(GithubClientError::RequestError(e)),
        }
    }

    pub(crate) async fn merge_pull(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
    ) -> Result<(), GithubClientError> {
        let route = format!("{GITHUB_API_ROOT}/repos/{owner}/{repo}/pulls/{pull_number}/merge");
        match self.post(route, <Option<&()>>::None, None).await {
            Ok(r) => match r.status() {
                StatusCode::OK => Ok(()),
                _ => Err(GithubClientError::GithubError(r)),
            },
            Err(e) => Err(GithubClientError::RequestError(e)),
        }
        // /repos/{owner}/{repo}/pulls/{pull_number}/merge
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

    #[tokio::test]
    async fn jwt() {
        let config = get_config();
        let client = GithubClient::new(&config.access_token());

        println!("{}", client.generate_jwt());
    }
}
