use axum::{Router, routing::get};

use crate::state::AppState;

pub mod token;

pub fn get_v1_router() -> Router<AppState> {
    Router::new().route("/token/validate", get(token::validate_token))
}
