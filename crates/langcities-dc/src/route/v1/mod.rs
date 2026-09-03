use axum::Router;

use crate::state::AppState;

pub mod token;
pub mod users;

pub fn get_v1_router() -> Router<AppState> {
    Router::new()
        .merge(token::get_v1_token_router())
        .merge(users::get_v1_users_router())
}
