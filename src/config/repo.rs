use serde::Deserialize;

use super::tests_config::TestsConfig;

#[derive(Debug, Deserialize)]
pub(crate) struct RepoConfig {
    owner: String,
    secret: String,
    tests: Option<TestsConfig>,
}
