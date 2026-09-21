use axum::Router;

use crate::state::AppState;

pub mod entries;
pub mod token;
pub mod users;
pub mod vernaculars;

pub fn get_v1_router() -> Router<AppState> {
    Router::new()
        .merge(entries::get_v1_entries_router())
        .merge(token::get_v1_token_router())
        .merge(users::get_v1_users_router())
        .merge(vernaculars::get_v1_vernaculars_router())
}
