use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::Read,
    net::{Ipv4Addr, SocketAddrV4},
    sync::Arc,
};

use serde::{de::IntoDeserializer, Deserialize};

use crate::CONFIG;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    server: Option<ServerConfig>,
    repos: HashMap<String, RepoConfig>,
    github: GithubConfig,
    logging: Option<LoggingConfig>,
    database: Option<DatabaseConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct DatabaseConfig {
    path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct ServerConfig {
    host: Option<Ipv4Addr>,
    port: Option<u16>,
    ssl: Option<bool>,
}

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 8000;
const DEFAULT_DATABASE_PATH: &str = "./yad.db";

impl ServerConfig {
    pub(crate) fn get_addr(&self) -> SocketAddrV4 {
        let (ip, port) = (
            if let Some(h) = self.host {
                h
            } else {
                DEFAULT_HOST.parse().unwrap()
            },
            if let Some(p) = self.port {
                p
            } else {
                DEFAULT_PORT
            },
        );

        SocketAddrV4::new(ip, port)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: Some(DEFAULT_HOST.parse().unwrap()),
            port: Some(DEFAULT_PORT),
            ssl: Some(false),
        }
    }
}

impl Config {
    pub(crate) fn server(&self) -> ServerConfig {
        match &self.server {
            Some(s) => (*s).clone(),
            None => ServerConfig::default(),
        }
    }

    pub(crate) fn get_repo(&self, name: &str) -> Option<&RepoConfig> {
        self.repos.get(name)
    }

    pub(crate) fn access_token(&self) -> &String {
        &self.github.access_token
    }

    pub(crate) fn logging(&self) -> &Option<LoggingConfig> {
        &self.logging
    }

    pub(crate) fn database_path(&self) -> String {
        match &self.database {
            Some(db) => match &db.path {
                Some(p) => p.into(),
                None => DEFAULT_DATABASE_PATH.into(),
            },
            None => DEFAULT_DATABASE_PATH.into(),
        }
    }
}
const DEFAULT_CONFIG_FILE_NAME: &str = "yad.toml";

pub(crate) fn load_config(config_file: Option<&str>) -> Result<Config, Box<dyn Error>> {
    let file_name = if let Some(cf) = config_file {
        cf
    } else {
        DEFAULT_CONFIG_FILE_NAME
    };

    let mut f = File::open(file_name)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    Ok(toml::from_str::<Config>(buf.as_str())?)
}

pub(crate) fn get_config() -> Arc<Config> {
    CONFIG.clone()
}
#[derive(Debug, Deserialize)]
pub(crate) struct GithubConfig {
    access_token: String,
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RepoConfig {
    owner: String,
    secret: String,
}

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
