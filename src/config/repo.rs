use serde::Deserialize;

use super::tests_config::TestsConfig;

#[derive(Debug, Deserialize)]
pub(crate) struct RepoConfig {
    owner: String,
    secret: String,
    tests: Option<TestsConfig>,

    /// Check names on the Github workflow runs to wait for before merging
    checks: Option<Vec<String>>,
}
