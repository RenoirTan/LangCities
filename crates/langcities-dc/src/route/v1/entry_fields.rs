use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, patch, post},
};
use langcities_common_server::dto::request::RequestContext;

use crate::{
    dto::entry_fields::{
        CreateEntryFieldDto, EntryFieldAliasDto, EntryFieldDto, UpdateEntryFieldDto,
    },
    error::{DcAppError, DcAppErrorTrait},
    repo::entry_field::EntryFieldRepo,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/v1/entry_fields/{alias}",
    params(
        (
            "alias" = EntryFieldAliasDto,
            Path,
            description = "Unique entry identifier, such as its numeric ID or alias"
        )
    ),
    responses(
        (status = 200, body = EntryFieldDto, description = "entry details")
    )
)]
#[axum::debug_handler]
pub async fn get_entry_field(
    Path(alias): Path<EntryFieldAliasDto>,
    request_context: RequestContext,
    State(state): State<AppState>,
) -> Result<Json<EntryFieldDto>, DcAppError> {
    EntryFieldRepo::from_state(state.clone())
        .get_entry_field(&state.db, alias.0.clone(), request_context)
        .await?
        .map(|f| Json(f.into()))
        .ok_or_else(|| DcAppError::not_found(format!("field {} not found", alias)))
}

#[utoipa::path(
    post,
    path = "/v1/entry_fields",
    request_body = CreateEntryFieldDto,
    responses(
        (status = 200, body = EntryFieldDto, description = "new entry details"),
        (status = 409, description = "entry field conflicts with existing data")
    )
)]
#[axum::debug_handler]
pub async fn create_entry_field(
    State(state): State<AppState>,
    request_context: RequestContext,
    Json(dto): Json<CreateEntryFieldDto>,
) -> Result<Json<EntryFieldDto>, DcAppError> {
    EntryFieldRepo::from_state(state.clone())
        .create_entry_field(&state.db, dto, request_context)
        .await
        .map(|f| Json(f.into()))
}

#[utoipa::path(
    patch,
    path = "/v1/entry_fields/{alias}",
    params(
        (
            "alias" = EntryFieldAliasDto,
            Path,
            description = "Unique entry field identifier, such as its numeric ID or alias"
        )
    ),
    request_body = UpdateEntryFieldDto,
    responses(
        (status = 200, body = EntryFieldDto, description = "entry field details")
    )
)]
#[axum::debug_handler]
pub async fn update_entry_field(
    Path(alias): Path<EntryFieldAliasDto>,
    request_context: RequestContext,
    State(state): State<AppState>,
    Json(dto): Json<UpdateEntryFieldDto>,
) -> Result<Json<EntryFieldDto>, DcAppError> {
    EntryFieldRepo::from_state(state.clone())
        .update_entry_field(&state.db, alias.0.clone(), dto, request_context)
        .await?
        .map(|f| Json(f.into()))
        .ok_or_else(|| DcAppError::not_found(format!("field {} not found", alias)))
}

#[utoipa::path(
    delete,
    path = "/v1/entry_fields/{alias}",
    params(
        (
            "alias" = EntryFieldAliasDto,
            Path,
            description = "Unique entry field identifier, such as its numeric ID or alias"
        )
    ),
    responses(
        (status = 200, body = EntryFieldDto, description = "entry field details")
    )
)]
#[axum::debug_handler]
pub async fn delete_entry_field(
    Path(alias): Path<EntryFieldAliasDto>,
    request_context: RequestContext,
    State(state): State<AppState>,
) -> Result<Json<EntryFieldDto>, DcAppError> {
    EntryFieldRepo::from_state(state.clone())
        .delete_entry_field(&state.db, alias.0.clone(), request_context)
        .await?
        .map(|f| Json(f.into()))
        .ok_or_else(|| DcAppError::not_found(format!("field {} not found", alias)))
}

pub fn get_v1_entry_fields_router() -> Router<AppState> {
    Router::new()
        .route("/entry_fields/{alias}", get(get_entry_field))
        .route("/entry_fields", post(create_entry_field))
        .route("/entry_fields/{alias}", patch(update_entry_field))
        .route("/entry_fields/{alias}", delete(delete_entry_field))
}
