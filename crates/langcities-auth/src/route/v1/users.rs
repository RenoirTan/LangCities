use crate::{
    dto::users::{ManyUserAliasDto, ManyUsersDto},
    entity::*,
    error::AuthAppErrorTrait,
    session::SessionUserWrapper,
};
use axum::{
    Json,
    extract::{Path, State},
};
use axum_extra::extract::Query;
use langcities_lcdcdsl::component::Alias;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};

use crate::{
    dto::users::{UserAliasDto, UserDto},
    error::AuthAppError,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/v1/users/{alias}",
    params(UserAliasDto),
    responses(
        (status = 200, body = UserDto, description = "user details")
    )
)]
#[axum::debug_handler]
pub async fn get_user(
    Path(alias): Path<UserAliasDto>,
    State(state): State<AppState>,
) -> Result<Json<UserDto>, AuthAppError> {
    let sql = match &alias.0 {
        Alias::Id(id) => users::Entity::find_by_id(**id),
        Alias::Slug(username) => users::Entity::find_by_username(&**username),
    };
    sql.one(&state.db)
        .await
        .map(|o| {
            o.map(|m| Json(m.into())).ok_or_else(|| {
                AuthAppError::not_found(Some(format!("{} not found", alias.0).into()))
            })
        })
        .map_err(|e| AuthAppError::database(Some(e.into())))
        .flatten()
}

#[utoipa::path(
    get,
    path = "/v1/users",
    params(ManyUserAliasDto),
    responses(
        (status = 200, body = ManyUsersDto, description = "user details")
    )
)]
pub async fn get_many_users(
    Query(query): Query<ManyUserAliasDto>,
    State(state): State<AppState>,
) -> Result<Json<ManyUsersDto>, AuthAppError> {
    if query.aliases.len() == 0 {
        return Ok(Json(ManyUsersDto { users: vec![] }));
    }
    let condition = query
        .aliases
        .into_iter()
        .fold(Condition::any(), |q, a| match a.0 {
            Alias::Id(id) => q.add(users::Column::Id.eq(*id)),
            Alias::Slug(username) => q.add(users::Column::Username.eq(&*username)),
        });
    let sql = users::Entity::find().filter(condition);
    sql.all(&state.db)
        .await
        .map(|ms| Json(ms.into_iter().collect()))
        .map_err(|e| AuthAppError::database(Some(e.into())))
}

#[utoipa::path(
    delete,
    path = "/v1/users",
    description = "must be logged in to use",
    responses(
        (status = 200, body = UserDto, description = "original user details")
    )
)]
pub async fn delete_user(
    session_user: SessionUserWrapper,
    State(state): State<AppState>,
) -> Result<Json<UserDto>, AuthAppError> {
    let user_session = session_user.get_user();
    let id = user_session
        .id
        .ok_or_else(|| AuthAppError::unauthorized(Some("not logged in".into())))?;
    let user: UserDto = users::Entity::delete_by_id(id)
        .exec_with_returning(&state.db)
        .await
        .map(|o| {
            o.map(|m| m.into()).ok_or_else(|| {
                AuthAppError::not_found(Some(format!("user {} not found", id).into()))
            })
        })
        .map_err(|e| AuthAppError::database(Some(e.into())))
        .flatten()?;
    session_user
        .delete()
        .await
        .map(|()| Json(user))
        .map_err(|e| AuthAppError::database(Some(e.into())))
}
