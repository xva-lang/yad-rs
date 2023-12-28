use std::{
    error::Error,
    fs::File,
    io::{self, Read},
    net::Ipv4Addr,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    server: Option<ServerConfig>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ServerConfig {
    host: Option<Ipv4Addr>,
    port: Option<u16>,
    ssl: Option<bool>,
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
