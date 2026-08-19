use axum::{Router, routing::post};

use crate::{route::v1::login::password_login, state::AppState};

pub mod login;

pub fn get_v1_router() -> Router<AppState> {
    let router = Router::new().route("/login/password", post(password_login));
    router
}
