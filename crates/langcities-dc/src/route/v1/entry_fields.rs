use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, patch, post},
};
use chrono::Utc;
use langcities_server::dto::request::RequestContext;

use crate::{
    api::state::ApiState,
    dto::entry_fields::{
        CreateEntryFieldDto, EntryFieldAliasDto, EntryFieldDto, FoundManyEntryFields,
        GetManyEntryFieldAliasDto, UpdateEntryFieldDto,
    },
    error::{DcAppError, DcAppErrorTrait},
    message::event::{EntryFieldUpdateEvent, EntryFieldValueChanged},
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
    State(state): State<ApiState>,
) -> Result<Json<EntryFieldDto>, DcAppError> {
    state
        .inner
        .entry_field_repo()
        .get_entry_field(&state.db, alias.0.clone().into(), request_context, None)
        .await?
        .map(|f| f.try_into())
        .ok_or_else(|| DcAppError::not_found_desc(format!("field {} not found", alias)))
        .flatten()
        .map(Json)
}

#[utoipa::path(
    get,
    path = "/v1/entry_fields",
    request_body = GetManyEntryFieldAliasDto,
    responses(
        (status = 200, body = FoundManyEntryFields, description = "entry details")
    )
)]
#[axum::debug_handler]
pub async fn get_entry_fields(
    request_context: RequestContext,
    State(state): State<ApiState>,
    Json(dto): Json<GetManyEntryFieldAliasDto>,
) -> Result<Json<FoundManyEntryFields>, DcAppError> {
    let aliases = dto.aliases.into_iter().map(|a| a.0.into());
    state
        .inner
        .entry_field_repo()
        .get_entry_fields(&state.db, aliases, request_context, None)
        .await
        .map(Json)
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
    State(state): State<ApiState>,
    request_context: RequestContext,
    Json(dto): Json<CreateEntryFieldDto>,
) -> Result<Json<EntryFieldDto>, DcAppError> {
    state
        .inner
        .entry_field_repo()
        .create_entry_field(&state.db, dto, request_context)
        .await
        .map(|f| f.try_into())
        .flatten()
        .map(Json)
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
    State(state): State<ApiState>,
    Json(dto): Json<UpdateEntryFieldDto>,
) -> Result<Json<EntryFieldDto>, DcAppError> {
    let (old, new) = state
        .inner
        .entry_field_repo()
        .update_entry_field_returning_original(
            &state.db,
            alias.0.clone().into(),
            dto,
            request_context,
            None,
        )
        .await?
        .ok_or_else(|| DcAppError::not_found_desc(format!("field {} not found", alias)))?;
    if old.override_value != new.override_value {
        let event = EntryFieldUpdateEvent {
            entry_field_id: old.id.into(),
            changed: EntryFieldValueChanged::Override,
            message_at: Utc::now(),
        };
        state
            .outer
            .api_mxs
            .efue_tx
            .send(event)
            .await
            .map_err(DcAppError::messaging_src)?;
    }
    Ok(Json(new.try_into()?))
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
    State(state): State<ApiState>,
) -> Result<Json<EntryFieldDto>, DcAppError> {
    state
        .inner
        .entry_field_repo()
        .delete_entry_field(&state.db, alias.0.clone().into(), request_context, None)
        .await?
        .map(|f| f.try_into())
        .ok_or_else(|| DcAppError::not_found_desc(format!("field {} not found", alias)))
        .flatten()
        .map(Json)
}

pub fn get_v1_entry_fields_router() -> Router<ApiState> {
    Router::new()
        .route("/entry_fields/{alias}", get(get_entry_field))
        .route("/entry_fields", post(create_entry_field))
        .route("/entry_fields/{alias}", patch(update_entry_field))
        .route("/entry_fields/{alias}", delete(delete_entry_field))
}
