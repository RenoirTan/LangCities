use axum::{Router, middleware};
use langcities_common::error::Error;
use langcities_jwt::axum::claims::parse_token_and_extend_state;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    openapi::ApiDoc, pre::seed::Seeder, state::AppState, util::setup::extract_current_user,
};

pub async fn api_main(state: AppState) -> Result<(), Error> {
    let bind_host = state.config.server.bind_host();
    if state.config.dc.seed_testing {
        Seeder::new(state.clone()).seed_testing().await?;
    }

    let swagger = SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi());
    let app = Router::new()
        .nest("/v1", crate::route::v1::get_v1_router())
        .merge(swagger)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            extract_current_user,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            parse_token_and_extend_state::<AppState>,
        ))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(bind_host).await?;
    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
