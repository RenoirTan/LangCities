use std::error::Error;

use axum::Router;
use sea_orm::Database;

use crate::config::{Config, PartialConfig};
use crate::pre::seed::Seeder;
use crate::route::v1::get_v1_router;
use crate::state::AppState;
use crate::util::password::PasswordChecker;

pub mod config;
pub mod dto;
pub mod entity;
pub mod error;
pub mod pre;
pub mod route;
pub mod state;
pub mod util;

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

    let opt = config.db.to_connection_options();
    let db = Database::connect(opt).await?;
    let pw_checker = PasswordChecker::default();
    let state = AppState::new(db, pw_checker);

    let seeder = Seeder::new(&state);
    if config.auth.seed_testing {
        seeder.seed_testing().await?;
    }

    let app = Router::new().nest("/v1", get_v1_router()).with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
