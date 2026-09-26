use crate::{
    dto::users::{GetUserParamsDto, GetUsersQueryDto, UserDto},
    error::{DcAppError, DcAppErrorTrait},
    repo::user::UserRepo,
    state::AppState,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use langcities_lcdcdsl::component::UserAlias;

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
    UserRepo
        .get_user_by_auth_user_id(&state.db, query.auth_user_id)
        .await
        .map(|u| {
            u.map(|u| Json(u.into()))
                .ok_or_else(|| DcAppError::not_found(format!("{} not found", query.auth_user_id)))
        })
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
    match &alias.0 {
        UserAlias::Id(id) => UserRepo.get_user_by_id(&state.db, **id).await,
        UserAlias::Slug(username) => {
            let auth_user_id = state.resolve_auth_user_id(&**username).await?;
            UserRepo
                .get_user_by_auth_user_id(&state.db, auth_user_id)
                .await
        }
    }
    .map(|u| {
        u.map(|u| Json(u.into()))
            .ok_or_else(|| DcAppError::not_found(format!("{} was not found", alias.0)))
    })
    .flatten()
}

pub fn get_v1_users_router() -> Router<AppState> {
    Router::new()
        .route("/users", get(get_user))
        .route("/users/{alias}", get(get_user_by_alias))
}
