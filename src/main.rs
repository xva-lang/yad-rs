use std::{default, error::Error};

use crate::config::{get_config, ServerConfig};
use axum::{routing::get, Router};

mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = match get_config(None) {
        Ok(c) => c,
        Err(e) => panic!(
            "Failed to read the configuration file. Extended error: {}",
            e
        ),
    };

    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // run our app with hyper, listening globally on port 3000
    let addr = config.server().get_addr();
    println!("Listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
