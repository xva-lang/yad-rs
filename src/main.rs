use std::{error::Error, sync::Arc, time::Duration};

use crate::config::{load_config, Config};
use async_sqlite::{rusqlite::params, PoolBuilder};
use axum::{
    routing::{get, post},
    Router,
};
use config::get_config;
use github::{model::User, GithubClient};
use logging::{error, info};
use model::{PullRequest, PullRequestStatus};

mod actions;
mod command;
mod config;
mod db;
mod github;
mod logging;
mod model;
mod queue;
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

const TESTS_ROOT_DIR: &str = "./test-queue";
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

    tokio::spawn(queue::queue_server());

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn fails() {
        panic!()
    }
}
