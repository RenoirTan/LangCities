use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, patch, post},
};
use langcities_server::dto::request::RequestContext;

use crate::{
    api::state::ApiState,
    dto::{
        users::UserDto,
        vernaculars::{
            CreateVernacularDto, UpdateVernacularDto, VernacularAliasDto, VernacularDto,
        },
    },
    error::{DcAppError, DcAppErrorTrait},
};

#[utoipa::path(
    get,
    path = "/v1/vernaculars/{alias}",
    params(
        (
            "alias" = VernacularAliasDto,
            Path,
            description = "Unique vernacular identifier, such as its numeric ID or alias"
        )
    ),
    responses(
        (status = 200, body = VernacularDto, description = "vernacular details")
    )
)]
#[axum::debug_handler]
pub async fn get_vernacular(
    Path(alias): Path<VernacularAliasDto>,
    request_context: RequestContext,
    State(state): State<ApiState>,
) -> Result<Json<VernacularDto>, DcAppError> {
    state
        .inner
        .vernacular_repo()
        .get_vernacular(&state.db, alias.0.clone().into(), request_context)
        .await
        .map(|m| {
            m.map(|m| m.try_into())
                .ok_or_else(|| DcAppError::not_found_desc(format!("{} not found", alias.0)))
                .flatten()
        })
        .flatten()
        .map(Json)
}

#[utoipa::path(
    post,
    path = "/v1/vernaculars",
    request_body = CreateVernacularDto,
    responses(
        (status = 200, body = VernacularDto, description = "new vernacular details"),
        (status = 409, description = "vernacular conflicts with existing data")
    )
)]
#[axum::debug_handler]
pub async fn create_vernacular(
    State(state): State<ApiState>,
    user: UserDto,
    request_context: RequestContext,
    Json(dto): Json<CreateVernacularDto>,
) -> Result<Json<VernacularDto>, DcAppError> {
    state
        .inner
        .vernacular_repo()
        .create_vernacular(&state.db, dto, user.auth_user_id, request_context)
        .await
        .map(|m| m.try_into())
        .flatten()
        .map(Json)
}

#[utoipa::path(
    patch,
    path = "/v1/vernaculars/{alias}",
    params(
        (
            "alias" = VernacularAliasDto,
            Path,
            description = "Unique vernacular identifier, such as its numeric ID or alias"
        )
    ),
    request_body = UpdateVernacularDto,
    responses(
        (status = 200, body = VernacularDto, description = "vernacular details")
    )
)]
#[axum::debug_handler]
pub async fn update_vernacular(
    Path(alias): Path<VernacularAliasDto>,
    request_context: RequestContext,
    State(state): State<ApiState>,
    Json(dto): Json<UpdateVernacularDto>,
) -> Result<Json<VernacularDto>, DcAppError> {
    state
        .inner
        .vernacular_repo()
        .update_vernacular(&state.db, alias.0.clone().into(), dto, request_context)
        .await
        .map(|o| {
            o.map(|m| m.try_into())
                .ok_or_else(|| DcAppError::not_found_desc(format!("{} not found", alias.0)))
                .flatten()
                .map(Json)
        })
        .flatten()
}

#[utoipa::path(
    delete,
    path = "/v1/vernaculars/{alias}",
    params(
        (
            "alias" = VernacularAliasDto,
            Path,
            description = "Unique vernacular identifier, such as its numeric ID or alias"
        )
    ),
    responses(
        (status = 200, body = VernacularDto, description = "vernacular details")
    )
)]
#[axum::debug_handler]
pub async fn delete_vernacular(
    Path(alias): Path<VernacularAliasDto>,
    request_context: RequestContext,
    State(state): State<ApiState>,
) -> Result<Json<VernacularDto>, DcAppError> {
    state
        .inner
        .vernacular_repo()
        .delete_vernacular(&state.db, alias.0.clone().into(), request_context)
        .await
        .map(|o| {
            o.map(|m| m.try_into())
                .ok_or_else(|| DcAppError::not_found_desc(format!("{} not found", alias.0)))
                .flatten()
        })
        .flatten()
        .map(Json)
}

pub fn get_v1_vernaculars_router() -> Router<ApiState> {
    Router::new()
        .route("/vernaculars/{alias}", get(get_vernacular))
        .route("/vernaculars", post(create_vernacular))
        .route("/vernaculars/{alias}", patch(update_vernacular))
        .route("/vernaculars/{alias}", delete(delete_vernacular))
}
