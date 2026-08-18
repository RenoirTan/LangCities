use std::{error::Error, time::Duration};

use axum::Router;
use axum::routing::post;
use sea_orm::{ConnectOptions, Database};

use crate::config::{Config, PartialConfig};
use crate::route::login::password_login;
use crate::state::AppState;

pub mod config;
pub mod dto;
pub mod entity;
pub mod error;
pub mod route;
pub mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    let partial_config = PartialConfig::collect()?;
    let config = Config::from_partial(partial_config)?;
    println!("config = {:#?}", config);

    let db_url = &config.db.url;

    println!("Connecting to database: {}", db_url);

    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(false) // disable SQLx logging
        .sqlx_logging_level(log::LevelFilter::Info);
    // .set_schema_search_path("auth_schema"); // set default Postgres schema

    let db = Database::connect(opt).await?;

    let app = Router::new()
        .route("/v1/login/password", post(password_login))
        .with_state(AppState::new(db));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;

    tracing::debug!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
