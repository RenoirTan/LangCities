use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, patch, post},
};
use langcities_common_server::dto::request::RequestContext;
use sea_orm::{ActiveModelTrait, DbErr, ModelTrait, TransactionError, TransactionTrait};

use crate::{
    dto::{
        entries::EntryAccessDto,
        entry_fields::{
            CreateEntryFieldDto, EntryFieldAccessDto, EntryFieldAliasDto, EntryFieldDto,
            UpdateEntryFieldDto,
        },
    },
    entity::entry_fields,
    error::{DcAppError, DcAppErrorTrait},
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
    EntryFieldAccessDto::read(alias.clone(), request_context)
        .resolve(&state.db, &state)
        .await
        .map(|o| {
            o.map(|m| Json(m.into()))
                .ok_or_else(|| DcAppError::not_found(format!("entry field {} not found", alias)))
        })
        .flatten()
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
    let entry_access = EntryAccessDto::write(dto.entry.clone(), request_context);

    let out = state
        .db
        .clone() // sea orm clone is cheap
        .transaction(|txn| {
            Box::pin(async move {
                let entry = entry_access
                    .resolve(txn, &state)
                    .await?
                    .ok_or_else(|| DcAppError::not_found(format!("{}", dto.entry)))?;
                let field = dto.to_active_model(&entry);
                match field.insert(txn).await {
                    Ok(model) => Ok(Json(model.into())),
                    Err(DbErr::RecordNotInserted) => {
                        Err(DcAppError::conflict(DbErr::RecordNotInserted))
                    }
                    Err(e) => Err(DcAppError::database(e)),
                }
            })
        })
        .await;

    out.map_err(|e| match e {
        TransactionError::Connection(e) => DcAppError::database(e),
        TransactionError::Transaction(e) => e,
    })
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
    let mut active: entry_fields::ActiveModel =
        EntryFieldAccessDto::write(alias.clone(), request_context)
            .resolve(&state.db, &state)
            .await
            .map(|o| o.ok_or_else(|| DcAppError::not_found(format!("{alias}"))))
            .flatten()?
            .into();
    dto.update_active_model(&mut active);
    active
        .update(&state.db)
        .await
        .map(|m| Json(EntryFieldDto::from(m)))
        .map_err(DcAppError::database)
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
    let model = EntryFieldAccessDto::delete(alias.clone(), request_context)
        .resolve(&state.db, &state)
        .await
        .map(|o| o.ok_or_else(|| DcAppError::not_found(format!("entry {} not found", alias))))
        .flatten()?;
    let dto = Json(EntryFieldDto::from(model.clone()));
    model
        .delete(&state.db)
        .await
        .map_err(DcAppError::database)?;
    Ok(dto)
}

pub fn get_v1_entry_fields_router() -> Router<AppState> {
    Router::new()
        .route("/entry_fields/{alias}", get(get_entry_field))
        .route("/entry_fields", post(create_entry_field))
        .route("/entry_fields/{alias}", patch(update_entry_field))
        .route("/entry_fields/{alias}", delete(delete_entry_field))
}
