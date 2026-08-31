use std::error::Error;

use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod config;
pub mod error;
pub mod openapi;
pub mod route;
pub mod state;
pub mod util;

use crate::config::{Config, PartialConfig};
use crate::openapi::ApiDoc;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    let partial_config = PartialConfig::collect()?;
    let config = Config::from_partial(partial_config)?;
    println!("{:#?}", config);

    let bind_host = config.server.bind_host();
    let state = AppState::create(config).await?;

    let swagger = SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi());
    let app = Router::new()
        .nest("/v1", route::v1::get_v1_router())
        .merge(swagger)
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(bind_host).await?;
    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
