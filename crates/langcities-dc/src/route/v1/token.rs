use crate::util::jwt::DcClaimsWrapper;
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

#[utoipa::path(get, path = "/v1/token/validate")]
#[axum::debug_handler]
pub async fn validate_token(_claims: DcClaimsWrapper) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "valid": true }))).into_response()
}
