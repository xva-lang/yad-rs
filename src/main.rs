use std::{error::Error, sync::Arc};

use crate::config::{load_config, Config};
use axum::{
    routing::{get, post},
    Router,
};
use config::get_config;
use github::{model::User, GithubClient};

mod actions;
mod command;
mod config;
mod db;
mod github;
mod logging;
mod model;
mod routes;

lazy_static::lazy_static! {
    static ref CONFIG: Arc<Config> = Arc::new(load_config(None).unwrap());
}
#[derive(Debug, Clone)]
struct AppState {
    app_user: User,
}

async fn get_current_user() -> Result<User, Box<dyn Error>> {
    let config = get_config();
    let client = GithubClient::new(config.access_token());
    let user: User = client.get_authenticated_user().await?;
    Ok(user)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = match load_config(None) {
        Ok(c) => c,
        Err(e) => panic!(
            "Failed to read the configuration file. Extended error: {}",
            e
        ),
    };

    let state = AppState {
        app_user: get_current_user().await?,
    };

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/github", post(routes::post_github))
        .with_state(state);

    let addr = config.server().get_addr();
    // println!("Listening on {addr}");
    logging::info(format!("Listening on {addr}"), Some(&config));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
