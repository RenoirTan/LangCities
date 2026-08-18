use std::{error::Error, time::Duration};

use sea_orm::{ConnectOptions, Database, EntityTrait};

use crate::config::{Config, PartialConfig};
use crate::entity::prelude::*;

pub mod config;
pub mod entity;

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

    let users = Users::find().all(&db).await?;

    println!("Users: {:?}", users);

    Ok(())
}
