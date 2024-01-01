use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct LoggingConfig {
    pub journalctl: Option<JournalctlLogging>,
    pub stdout: Option<StdoutLogging>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct JournalctlLogging {
    /// The value to use as the `SYSLOG_IDENTIFIER` for `journalctl`. If this value is `None`,
    /// the default value of `yad` is used.
    pub identifier: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StdoutLogging {}
