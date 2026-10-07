use futures::FutureExt;
use futures::stream::StreamExt;
use langcities_common::error::Error;
use langcities_signal::lc::{SignalKind, SignallerConfig};
use tokio::sync::oneshot::channel;

pub mod aio;
pub mod api;
pub mod config;
pub mod dto;
pub mod entity;
pub mod error;
pub mod hub;
pub mod manager;
pub mod message;
pub mod openapi;
pub mod pre;
pub mod repo;
pub mod route;
pub mod state;
pub mod util;
pub mod worker;

use crate::aio::aio_main;
use crate::api::api_main;
use crate::config::{Config, DcSubcommand, PartialConfig};
use crate::error::{DcAppError, DcAppErrorTrait};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    let mut signaller = SignallerConfig::default().create_signaller()?;
    let mut signal_stream = signaller
        .boxed_simple_stream()
        .ok_or_else(|| DcAppError::failed_init("could not get signal stream"))?;

    let result = 'outer: loop {
        let (stop_tx, stop_rx) = channel::<()>();
        let stop_rx = stop_rx.map(|r| r.unwrap_or(()));

        let partial_config = PartialConfig::collect()?;
        let config = Config::from_partial(partial_config)?;
        if config.dc.show_config_only {
            println!("{:#?}", config);
            return Ok(());
        }

        // &mut [`tokio::task::JoinHandle`] is cancel safe
        let mut app_handle = match config.dc.subcommand {
            DcSubcommand::Aio => tokio::spawn(aio_main(config, stop_rx)),
            DcSubcommand::Api => tokio::spawn(api_main(config, stop_rx)),
            _ => unimplemented!(),
        };

        // this inner loop is to continuously receive signals
        // and decide what to do with them
        'inner: loop {
            let signal = tokio::select! {
                signal = signal_stream.next() => signal.unwrap_or(SignalKind::Stop),
                // if app_handle finishes, usually indicates some panic in the app itself
                // exit the program just in case
                join_result = &mut app_handle => {
                    match join_result {
                        Ok(result) => break 'outer result,
                        Err(join_error) => break 'outer Err(join_error.into()),
                    }
                },
            };
            match signal {
                SignalKind::Ignore => continue,
                SignalKind::Stop => {
                    let _ = stop_tx.send(());
                    let join_result = app_handle.await;
                    match join_result {
                        Ok(result) => break 'outer result,
                        Err(join_error) => break 'outer Err(join_error.into()),
                    }
                }
                SignalKind::Restart => {
                    let _ = stop_tx.send(());
                    let join_result = app_handle.await;
                    match join_result {
                        Ok(Ok(())) => break 'inner,
                        Ok(Err(e)) => break 'outer Err(e),
                        Err(join_error) => break 'outer Err(join_error.into()),
                    }
                }
            }
        }
    };

    signaller.shutdown();
    result
}
