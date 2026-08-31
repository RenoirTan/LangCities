use axum::{Router, routing::post};

use crate::{route::v1::login::password_login, state::AppState};

pub mod login;
pub mod token;

pub fn get_v1_router() -> Router<AppState> {
    let router = Router::new()
        .route("/login/password", post(password_login))
        .route(
            "/token/access/generic",
            post(token::issue_generic_access_token),
        )
        .route("/token/access/dc", post(token::issue_dc_access_token));
    router
}
