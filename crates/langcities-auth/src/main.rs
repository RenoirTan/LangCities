use axum::Router;
use futures::StreamExt;
use langcities_common::error::Error;
use langcities_signal::lc::{SignalKind, SignallerConfig};
use tokio::sync::oneshot::channel;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::{Config, PartialConfig};
use crate::error::{AuthAppError, AuthAppErrorTrait};
use crate::openapi::ApiDoc;
use crate::pre::seed::Seeder;
use crate::route::v1::get_v1_router;
use crate::session::build_session_layer;
use crate::state::AppState;

pub mod config;
pub mod dto;
pub mod entity;
pub mod error;
pub mod openapi;
pub mod pre;
pub mod route;
pub mod session;
pub mod state;
pub mod util;

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Hello, world!");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    let mut signaller = SignallerConfig::default().create_signaller()?;
    let mut shutdown_stream = signaller
        .boxed_simple_stream()
        .ok_or_else(|| AuthAppError::failed_init("could not get shutdown signal"))?;

    let result = 'outer: loop {
        let (stop_tx, stop_rx) = channel::<()>();
        let shutdown_signal = Box::pin(async move {
            let _ = stop_rx.await;
            ()
        });

        let partial_config = PartialConfig::collect()?;
        let config = Config::from_partial(partial_config)?;
        println!("config = {:#?}", config);
        let db_url = &config.db.url;
        println!("Connecting to database: {}", db_url);

        let state = AppState::create(config.clone()).await?;

        let seeder = Seeder::new(&state);
        if config.auth.seed_testing {
            seeder.seed_testing().await?;
        }

        let session_layer = build_session_layer(&config.auth, &state.db).await?;

        let swagger =
            SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi());
        let app = Router::new()
            .nest("/v1", get_v1_router())
            .merge(swagger)
            .layer(session_layer)
            .with_state(state);
        let listener = tokio::net::TcpListener::bind(config.server.bind_host()).await?;
        tracing::debug!("listening on {}", listener.local_addr()?);
        let mut app_handle = tokio::spawn(
            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal)
                .into_future(),
        );

        'inner: loop {
            let signal = tokio::select! {
                signal = shutdown_stream.next() => {
                    signal.unwrap_or(SignalKind::Stop)
                },
                join_result = &mut app_handle => {
                    match join_result {
                        Ok(result) => break 'outer result.map_err(|e| e.into()),
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
                        Ok(result) => break 'outer result.map_err(|e| e.into()),
                        Err(join_error) => break 'outer Err(join_error.into()),
                    }
                }
                SignalKind::Restart => {
                    let _ = stop_tx.send(());
                    let join_result = app_handle.await;
                    match join_result {
                        Ok(Ok(())) => break 'inner,
                        Ok(Err(e)) => break 'outer Err(e.into()),
                        Err(join_error) => break 'outer Err(join_error.into()),
                    }
                }
            }
        }
    };

    signaller.shutdown();
    result
}
