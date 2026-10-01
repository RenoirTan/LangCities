use langcities_common::error::Error;

pub mod aio;
pub mod api;
pub mod config;
pub mod dto;
pub mod entity;
pub mod error;
pub mod manager;
pub mod message;
pub mod openapi;
pub mod pre;
pub mod repo;
pub mod route;
pub mod state;
pub mod util;
pub mod worker;

use crate::api::api_main;
use crate::config::{Config, DcSubcommand, PartialConfig};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    let partial_config = PartialConfig::collect()?;
    let config = Config::from_partial(partial_config)?;
    println!("{:#?}", config);

    match config.dc.subcommand {
        DcSubcommand::Api => api_main(config).await,
        _ => unimplemented!(),
    }
}
