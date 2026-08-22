use std::error::Error;

use axum::Router;
use sea_orm::Database;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::{Config, PartialConfig};
use crate::openapi::ApiDoc;
use crate::pre::seed::Seeder;
use crate::route::v1::get_v1_router;
use crate::session::build_session_layer;
use crate::state::AppState;
use crate::util::password::PasswordChecker;

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

    let session_layer = build_session_layer(&config.auth, &state.db).await?;

    let swagger = SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi());
    let app = Router::new()
        .nest("/v1", get_v1_router())
        .merge(swagger)
        .layer(session_layer)
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(config.server.bind_host()).await?;
    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
