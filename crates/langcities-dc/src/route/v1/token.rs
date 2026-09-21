use crate::state::AppState;
use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use langcities_jwt::payload::Claims;

#[utoipa::path(
    get,
    path = "/v1/token/validate",
    description = "validate access token, must be placed in bearer header, do not rely on the response body for anything, the only thing stable is the http status"
)]
#[axum::debug_handler]
pub async fn validate_token(claims: Claims, State(_state): State<AppState>) -> impl IntoResponse {
    Json(claims)
}

pub fn get_v1_token_router() -> Router<AppState> {
    Router::new().route("/token/validate", get(validate_token))
}
