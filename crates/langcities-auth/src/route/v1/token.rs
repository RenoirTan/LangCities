use axum::{Json, extract::State};

use crate::{
    dto::token::AccessTokenResponseDto,
    error::AuthAppError,
    session::SessionUserWrapper,
    state::AppState,
    util::jwt::{Access, Authorization, Microservice},
};

#[utoipa::path(post, path = "/v1/token/access/generic")]
#[axum::debug_handler]
pub async fn issue_generic_access_token(
    State(state): State<AppState>,
    session_user: SessionUserWrapper,
) -> Result<Json<AccessTokenResponseDto>, AuthAppError> {
    let access = Access::new(
        Authorization::session(session_user.get_user().clone()),
        Microservice::Generic,
    );
    access.mint(&state).map(|d| Json(d))
}

#[utoipa::path(post, path = "/v1/token/access/dc")]
#[axum::debug_handler]
pub async fn issue_dc_access_token(
    State(state): State<AppState>,
    session_user: SessionUserWrapper,
) -> Result<Json<AccessTokenResponseDto>, AuthAppError> {
    println!("{:?}", session_user);
    let access = Access::new(
        Authorization::session(session_user.get_user().clone()),
        Microservice::Dc,
    );
    access.mint(&state).map(|d| Json(d))
}
