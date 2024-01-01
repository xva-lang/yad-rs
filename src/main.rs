use axum::{
    routing::{get, post},
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use std::{error::Error, net::TcpListener, sync::Arc, time::Duration};

use crate::config::{load_config, Config};

use config::get_config;
use entity::pull_requests::{Entity as PullRequest, PullRequestStatus};
use github::{model::User, GithubClient};
use logging::{error, info};

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

    // // Initialise the database
    // db::create_db().await?;
    db::apply_migrations().await?;

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/github", post(routes::post_github))
        .with_state(state);

    tokio::spawn(queue::queue_server());

    start(app).await;

    Ok(())
}

async fn start(app: Router) {
    let config = get_config();

    let addr = config.server().get_addr();
    logging::info(format!("Listening on {addr}"), Some(&config));

    let ssl_config = match config.server {
        Some(ref s) => match s.ssl {
            Some(ref ssl) => Some(ssl.to_pem_file().await.unwrap()),
            None => None,
        },
        None => None,
    };

    if let Some(ssl) = ssl_config {
        logging::info(format!("HTTPS is enabled"), Some(&config));
        axum_server::tls_rustls::bind_rustls(std::net::SocketAddr::V4(addr), ssl)
            .serve(app.into_make_service())
            .await
            .unwrap()
    } else {
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }
}

#[cfg(test)]
mod tests {}
