use crate::{
    dto::users::{GetUserParamsDto, GetUsersQueryDto, UserDto},
    entity::dc_users,
    error::{DcAppError, DcAppErrorTrait},
    state::AppState,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use langcities_lcdcdsl::component::UserAlias;
use sea_orm::EntityTrait;

#[utoipa::path(
    get,
    path = "/v1/users",
    params(GetUsersQueryDto),
    responses(
        (status = 200, body = UserDto, description = "user details")
    )
)]
#[axum::debug_handler]
pub async fn get_user(
    Query(query): Query<GetUsersQueryDto>,
    State(state): State<AppState>,
) -> Result<Json<UserDto>, DcAppError> {
    dc_users::Entity::find_by_auth_user_id(query.auth_user_id)
        .one(&state.db)
        .await
        .map(|u| {
            u.map(|u| Json(u.into()))
                .ok_or_else(|| DcAppError::not_found(format!("{} not found", query.auth_user_id)))
        })
        .map_err(DcAppError::database)
        .flatten()
}

#[utoipa::path(
    get,
    path = "/v1/users/{alias}",
    params(("alias" = GetUserParamsDto, Path)),
    responses(
        (status = 200, body = UserDto, description = "user details")
    )
)]
#[axum::debug_handler]
pub async fn get_user_by_alias(
    Path(alias): Path<GetUserParamsDto>,
    State(state): State<AppState>,
) -> Result<Json<UserDto>, DcAppError> {
    let user = match alias.0.clone() {
        UserAlias::Id(id) => dc_users::Entity::find_by_id(*id).one(&state.db).await,
        UserAlias::Slug(username) => {
            let auth_id = state.resolve_auth_user_id(&**username).await?;
            dc_users::Entity::find_by_auth_user_id(auth_id)
                .one(&state.db)
                .await
        }
    };
    match user {
        Ok(Some(user)) => Ok(Json(user.into())),
        Ok(None) => Err(DcAppError::not_found(format!(
            "user '{}' not found",
            alias.0
        ))),
        Err(e) => Err(DcAppError::database(e)),
    }
}

pub fn get_v1_users_router() -> Router<AppState> {
    Router::new()
        .route("/users", get(get_user))
        .route("/users/{alias}", get(get_user_by_alias))
}
