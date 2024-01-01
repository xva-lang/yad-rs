use std::time::Duration;

use crate::config::get_config;
use migration::MigratorTrait;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};

pub(crate) async fn orm() -> Result<(), DbErr> {
    let config = get_config();
    let mut opt = ConnectOptions::new(format!("sqlite://{}?mode=rwc", config.database_path()));
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info)
        .set_schema_search_path("my_schema"); // Setting default PostgreSQL schema

    let db = Database::connect(opt).await?;

    Ok(())
}

pub(crate) async fn apply_migrations() -> Result<(), DbErr> {
    let config = get_config();
    let mut connect_options =
        ConnectOptions::new(format!("sqlite://{}?mode=rwc", config.database_path()));
    connect_options
        .max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info)
        .set_schema_search_path("my_schema"); // Setting default PostgreSQL schema

    let db = Database::connect(connect_options).await?;

    Ok(migration::Migrator::up(&db, None).await?)
}

pub(crate) async fn get_db() -> Result<DatabaseConnection, DbErr> {
    let config = get_config();
    let mut connect_options =
        ConnectOptions::new(format!("sqlite://{}?mode=rwc", config.database_path()));
    connect_options
        .max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info)
        .set_schema_search_path("my_schema"); // Setting default PostgreSQL schema

    Ok(Database::connect(connect_options).await?)
}
