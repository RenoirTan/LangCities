use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, patch, post},
};
use langcities_common_server::dto::request::RequestContext;

use crate::{
    dto::vernaculars::{
        CreateVernacularDto, UpdateVernacularDto, VernacularAliasDto, VernacularDto,
    },
    entity::dc_users,
    error::{DcAppError, DcAppErrorTrait},
    repo::vernacular::VernacularRepo,
    state::AppState,
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
    State(state): State<AppState>,
) -> Result<Json<VernacularDto>, DcAppError> {
    VernacularRepo::new(state.clone())
        .get_vernacular(&state.db, alias.0.clone(), request_context)
        .await
        .map(|m| {
            m.map(|m| Json(m.into()))
                .ok_or_else(|| DcAppError::not_found(format!("{} not found", alias.0)))
        })
        .flatten()
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
    State(state): State<AppState>,
    user: dc_users::Model,
    request_context: RequestContext,
    Json(dto): Json<CreateVernacularDto>,
) -> Result<Json<VernacularDto>, DcAppError> {
    VernacularRepo::new(state.clone())
        .create_vernacular(&state.db, dto, user.auth_user_id, request_context)
        .await
        .map(|m| Json(m.into()))
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
    State(state): State<AppState>,
    Json(dto): Json<UpdateVernacularDto>,
) -> Result<Json<VernacularDto>, DcAppError> {
    VernacularRepo::new(state.clone())
        .update_vernacular(&state.db, alias.0.clone(), dto, request_context)
        .await
        .map(|o| {
            o.map(|m| Json(m.into()))
                .ok_or_else(|| DcAppError::not_found(format!("{} not found", alias.0)))
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
    State(state): State<AppState>,
) -> Result<Json<VernacularDto>, DcAppError> {
    VernacularRepo::new(state.clone())
        .delete_vernacular(&state.db, alias.0.clone(), request_context)
        .await
        .map(|o| {
            o.map(|m| Json(m.into()))
                .ok_or_else(|| DcAppError::not_found(format!("{} not found", alias.0)))
        })
        .flatten()
}

pub fn get_v1_vernaculars_router() -> Router<AppState> {
    Router::new()
        .route("/vernaculars/{alias}", get(get_vernacular))
        .route("/vernaculars", post(create_vernacular))
        .route("/vernaculars/{alias}", patch(update_vernacular))
        .route("/vernaculars/{alias}", delete(delete_vernacular))
}
