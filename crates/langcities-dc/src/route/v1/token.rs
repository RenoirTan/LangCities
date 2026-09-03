use crate::state::AppState;
use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use langcities_jwt::payload::Claims;
use serde_json::json;

#[utoipa::path(get, path = "/v1/token/validate")]
#[axum::debug_handler]
pub async fn validate_token(_claims: Claims, State(_state): State<AppState>) -> impl IntoResponse {
    Json(json!({ "valid": true }))
}

pub fn get_v1_token_router() -> Router<AppState> {
    Router::new().route("/token/validate", get(validate_token))
}
