use std::{
    error::Error,
    fs::File,
    io::{self, Read},
    net::{Ipv4Addr, SocketAddrV4, ToSocketAddrs},
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    server: Option<ServerConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct ServerConfig {
    host: Option<Ipv4Addr>,
    port: Option<u16>,
    ssl: Option<bool>,
}

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 8000;

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
}
const DEFAULT_CONFIG_FILE_NAME: &str = "yad.toml";

pub(crate) fn get_config(config_file: Option<&str>) -> Result<Config, Box<dyn Error>> {
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
