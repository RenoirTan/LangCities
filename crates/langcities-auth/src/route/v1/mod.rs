use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::{
    route::v1::{
        login::password_login,
        users::{delete_user, get_many_users, get_user},
    },
    state::AppState,
};

pub mod login;
pub mod register;
pub mod token;
pub mod users;

pub fn get_v1_router() -> Router<AppState> {
    let router = Router::new()
        .route("/login/password", post(password_login))
        .route(
            "/token/access/generic",
            post(token::issue_generic_access_token),
        )
        .route("/token/access/dc", post(token::issue_dc_access_token))
        .route("/register", post(register::register))
        .route("/users/{alias}", get(get_user))
        .route("/users", get(get_many_users))
        .route("/users", delete(delete_user));
    router
}
