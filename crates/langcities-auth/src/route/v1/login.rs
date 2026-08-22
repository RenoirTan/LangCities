use axum::{Json, extract::State};
use tower_sessions::Session;

use crate::{
    dto::login::{PasswordLoginDto, PasswordLoginResponseDto},
    entity::prelude::*,
    error::{AuthAppError, AuthAppErrorResponseDto, AuthAppErrorTrait},
    session::SessionWrapper,
    state::AppState,
};

#[utoipa::path(post, path = "/v1/login/password")]
#[axum::debug_handler]
pub async fn password_login(
    State(state): State<AppState>,
    session: Session,
    Json(dto): Json<PasswordLoginDto>,
) -> Result<Json<PasswordLoginResponseDto>, AuthAppErrorResponseDto> {
    let user = Users::find_by_username(&dto.username)
        .one(&state.db)
        .await
        .map_err(|e| AuthAppError::database(Some(e.into())).to_response())?;
    let session = SessionWrapper::new(session);
    if let Some(user) = user {
        if let Some(hashed) = user.password_hash {
            let ok = state
                .pw_checker
                .verify_password(&dto.password, &hashed)
                .await
                .map_err(|e| e.to_response())?;
            if ok {
                session
                    .insert("user_id", user.id)
                    .await
                    .map_err(|e| AuthAppError::failed_session(Some(e.into())).to_response())?;
                return Ok(Json(PasswordLoginResponseDto {
                    message: "ok".into(),
                }));
            }
        }
    }
    Err(AuthAppError::invalid_credentials(Some(
        "Could not find any user with this combination of user and password.".into(),
    ))
    .to_response())
}
