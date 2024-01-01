use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct GithubConfig {
    pub access_token: String,
    pub oauth: GithubOauthConfig,
    pub app: GithubAppConfig,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GithubAppConfig {
    pub app_id: String,
    client_id: String,
    client_secret: String,
    pub private_key_file: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GithubOauthConfig {
    client_id: String,
    client_secret: String,
}
