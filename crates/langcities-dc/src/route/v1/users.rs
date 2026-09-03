use crate::{
    dto::users::{UserDto, UsersGetQueryDto},
    entity::dc_users,
    error::{DcAppError, DcAppErrorTrait},
    state::AppState,
};
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};

#[utoipa::path(get, path = "/v1/users/get", params(UsersGetQueryDto))]
#[axum::debug_handler]
pub async fn get_user(
    Query(query): Query<UsersGetQueryDto>,
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

pub fn get_v1_users_router() -> Router<AppState> {
    Router::new().route("/users/get", get(get_user))
}
