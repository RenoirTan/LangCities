use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, post},
};
use langcities_common_server::dto::request::RequestContext;

use crate::{
    dto::entries::{CreateEntryDto, EntryAliasDto, EntryDto},
    error::{DcAppError, DcAppErrorTrait},
    repo::entry::EntryRepo,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/v1/entries/{alias}",
    params(
        (
            "alias" = EntryAliasDto,
            Path,
            description = "Unique entry identifier, such as its numeric ID or alias"
        )
    ),
    responses(
        (status = 200, body = EntryDto, description = "entry details")
    )
)]
#[axum::debug_handler]
pub async fn get_entry(
    Path(alias): Path<EntryAliasDto>,
    request_context: RequestContext,
    State(state): State<AppState>,
) -> Result<Json<EntryDto>, DcAppError> {
    EntryRepo::from_state(state.clone())
        .get_entry(&state.db, alias.0.clone(), request_context)
        .await?
        .map(|m| Json(m.into()))
        .ok_or_else(|| DcAppError::not_found(format!("entry {} not found", alias)))
}

#[utoipa::path(
    post,
    path = "/v1/entries",
    request_body = CreateEntryDto,
    responses(
        (status = 200, body = EntryDto, description = "new entry details"),
        (status = 409, description = "entry conflicts with existing data")
    )
)]
#[axum::debug_handler]
pub async fn create_entry(
    State(state): State<AppState>,
    request_context: RequestContext,
    Json(dto): Json<CreateEntryDto>,
) -> Result<Json<EntryDto>, DcAppError> {
    EntryRepo::from_state(state.clone())
        .create_entry(&state.db, dto, request_context)
        .await
        .map(|m| Json(m.into()))
}

/*
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
    let mut active: vernaculars::ActiveModel =
        VernacularAccessDto::write(alias.clone(), request_context)
            .resolve(&state.db, &state)
            .await
            .map(|o| o.ok_or_else(|| DcAppError::not_found(format!("{alias}"))))
            .flatten()?
            .into();
    dto.update_active_model(&mut active);
    active
        .update(&state.db)
        .await
        .map(|m| Json(VernacularDto::from(m)))
        .map_err(DcAppError::database)
}
*/

#[utoipa::path(
    delete,
    path = "/v1/entries/{alias}",
    params(
        (
            "alias" = EntryAliasDto,
            Path,
            description = "Unique entry identifier, such as its numeric ID or alias"
        )
    ),
    responses(
        (status = 200, body = EntryDto, description = "entry details")
    )
)]
#[axum::debug_handler]
pub async fn delete_entry(
    Path(alias): Path<EntryAliasDto>,
    request_context: RequestContext,
    State(state): State<AppState>,
) -> Result<Json<EntryDto>, DcAppError> {
    EntryRepo::from_state(state.clone())
        .delete_entry(&state.db, alias.0.clone(), request_context)
        .await?
        .map(|m| Json(m.into()))
        .ok_or_else(|| DcAppError::not_found(format!("entry {} not found", alias)))
}

pub fn get_v1_entries_router() -> Router<AppState> {
    Router::new()
        .route("/entries/{alias}", get(get_entry))
        .route("/entries", post(create_entry))
        .route("/entries/{alias}", delete(delete_entry))
}
