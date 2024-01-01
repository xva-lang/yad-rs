use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct TestsConfig {
    custom: Option<CustomTestConfig>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CargoTestConfig {}

#[derive(Debug, Deserialize)]
pub(crate) struct CustomTestConfig {
    command: String,
}
