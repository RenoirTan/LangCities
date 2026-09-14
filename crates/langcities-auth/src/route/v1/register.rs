use axum::{Json, extract::State};
use chrono::Utc;
use sea_orm::{ActiveValue, EntityTrait, TryInsertResult};

use crate::{
    dto::{register::RegisterDto, user::UserDto},
    entity::users,
    error::{AuthAppError, AuthAppErrorTrait},
    state::AppState,
};

#[utoipa::path(
    post,
    path = "/v1/register",
    request_body = RegisterDto,
    responses(
        (status = 200, body = UserDto, description = "successful registration")
    )
)]
#[axum::debug_handler]
pub async fn register(
    State(state): State<AppState>,
    Json(dto): Json<RegisterDto>,
) -> Result<Json<UserDto>, AuthAppError> {
    let password_hash = state.pw_checker.hash_password(dto.password).await?;
    let now = Utc::now();
    let user = users::ActiveModel {
        username: ActiveValue::Set(dto.username),
        password_hash: ActiveValue::Set(Some(password_hash)),
        created_at: ActiveValue::Set(now.clone()),
        updated_at: ActiveValue::Set(now),
        ..Default::default()
    };
    match users::Entity::insert(user)
        .on_conflict_do_nothing_on([users::Column::Username])
        .exec(&state.db)
        .await
    {
        Ok(res) => match res {
            TryInsertResult::Empty => panic!("Not supposed to happen"),
            TryInsertResult::Conflicted => Err(AuthAppError::unauthorized(Some(
                "conflicting usernames".into(),
            ))),
            TryInsertResult::Inserted(i) => match users::Entity::find_by_id(i.last_insert_id)
                .one(&state.db)
                .await
            {
                Ok(Some(user)) => Ok(Json(user.into())),
                Ok(None) => Err(AuthAppError::other(Some("unsuccessful insert".into()))),
                Err(e) => Err(AuthAppError::database(Some(e.into()))),
            },
        },
        Err(e) => Err(AuthAppError::database(Some(e.into()))),
    }
}
