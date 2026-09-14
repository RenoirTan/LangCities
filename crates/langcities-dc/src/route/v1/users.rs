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
use langcities_lcdcdsl::component::Alias;
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
            u.map(|u| Json(u.into())).ok_or_else(|| {
                DcAppError::not_found(Some(format!("{} not found", query.auth_user_id).into()))
            })
        })
        .map_err(|e| DcAppError::database(Some(e.into())))
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
        Alias::Id(id) => dc_users::Entity::find_by_id(*id).one(&state.db).await,
        Alias::Slug(_username) => {
            return Err(DcAppError::bad_request(Some(
                "username not supported yet".into(),
            )));
        }
    };
    match user {
        Ok(Some(user)) => Ok(Json(user.into())),
        Ok(None) => Err(DcAppError::not_found(Some(
            format!("user '{}' not found", alias.0).into(),
        ))),
        Err(e) => Err(DcAppError::database(Some(e.into()))),
    }
}

pub fn get_v1_users_router() -> Router<AppState> {
    Router::new()
        .route("/users", get(get_user))
        .route("/users/{alias}", get(get_user_by_alias))
}
