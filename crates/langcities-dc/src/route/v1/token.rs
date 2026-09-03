use crate::{state::AppState, util::jwt::DcClaimsWrapper};
use axum::{Json, Router, response::IntoResponse, routing::get};
use serde_json::json;

#[utoipa::path(get, path = "/v1/token/validate")]
#[axum::debug_handler]
pub async fn validate_token(_claims: DcClaimsWrapper) -> impl IntoResponse {
    Json(json!({ "valid": true }))
}

pub fn get_v1_token_router() -> Router<AppState> {
    Router::new().route("/token/validate", get(validate_token))
}
