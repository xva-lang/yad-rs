use std::error::Error;

use crate::config::get_config;
use axum::{
    routing::{get, post},
    Router,
};
use github::model::User;

mod actions;
mod command;
mod config;
mod github;
mod logging;
mod routes;

#[derive(Debug, Clone)]
struct AppState {
    app_user: User,
}

async fn get_current_user() -> Result<User, Box<dyn Error>> {
    let client = octocrab::instance();
    let user: User = client.get("/user", <Option<&str>>::None).await?;
    Ok(user)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = match get_config(None) {
        Ok(c) => c,
        Err(e) => panic!(
            "Failed to read the configuration file. Extended error: {}",
            e
        ),
    };

    octocrab::initialise(
        octocrab::Octocrab::builder()
            .personal_token(config.access_token().to_string())
            .build()
            .unwrap(),
    );

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
