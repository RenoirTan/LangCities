use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use langcities_common_server::dto::request::RequestContext;
use sea_orm::{
    ActiveModelTrait, ActiveValue, DbErr, IntoActiveModel, TransactionError, TransactionTrait,
};

use crate::{
    dto::{
        entries::{CreateEntryDto, EntryAccessDto, EntryAliasDto, EntryDto},
        vernaculars::VernacularAccessDto,
    },
    error::{DcAppError, DcAppErrorTrait},
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
    EntryAccessDto::read(alias.clone(), request_context)
        .resolve(&state.db, &state)
        .await
        .map(|o| {
            o.map(|m| Json(m.into())).ok_or_else(|| {
                DcAppError::not_found(Some(format!("entry {} not found", alias).into()))
            })
        })
        .flatten()
}

#[utoipa::path(
    post,
    path = "/v1/entries",
    request_body = CreateEntryDto,
    responses(
        (status = 200, body = EntryDto, description = "new vernacular details")
    )
)]
#[axum::debug_handler]
pub async fn create_entry(
    State(state): State<AppState>,
    request_context: RequestContext,
    Json(dto): Json<CreateEntryDto>,
) -> Result<Json<EntryDto>, DcAppError> {
    let vernacular_alias = dto.vernacular.clone();
    let vernacular_access = VernacularAccessDto::write(dto.vernacular.clone(), request_context);

    let out = state
        .db
        .clone() // sea orm clone is cheap
        .transaction(|txn| {
            Box::pin(async move {
                let vernacular =
                    vernacular_access
                        .resolve(txn, &state)
                        .await?
                        .ok_or_else(|| {
                            DcAppError::not_found(Some(format!("{}", vernacular_alias).into()))
                        })?;
                let entry = dto.to_active_model(&vernacular);
                let next_index = vernacular.next_entry_id + 1;
                let response: Json<EntryDto> = match entry.insert(txn).await {
                    Ok(model) => Json(model.into()),
                    Err(DbErr::RecordNotInserted) => {
                        return Err(DcAppError::bad_request(Some(
                            DbErr::RecordNotInserted.into(),
                        )));
                    }
                    Err(e) => return Err(DcAppError::database(Some(e.into()))),
                };
                let mut v = vernacular.into_active_model();
                v.next_entry_id = ActiveValue::Set(next_index);
                v.update(txn)
                    .await
                    .map(|_| response)
                    .map_err(|e| DcAppError::database(Some(e.into())))
            })
        })
        .await;

    out.map_err(|e| match e {
        TransactionError::Connection(e) => DcAppError::database(Some(e.into())),
        TransactionError::Transaction(e) => e,
    })
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
            .map(|o| o.ok_or_else(|| DcAppError::not_found(Some(format!("{alias}").into()))))
            .flatten()?
            .into();
    dto.update_active_model(&mut active);
    active
        .update(&state.db)
        .await
        .map(|m| Json(VernacularDto::from(m)))
        .map_err(|e| DcAppError::database(Some(e.into())))
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
    let model = VernacularAccessDto::delete(alias.clone(), request_context)
        .resolve(&state.db, &state)
        .await
        .map(|o| {
            o.ok_or_else(|| {
                DcAppError::not_found(Some(format!("vernacular {} not found", alias).into()))
            })
        })
        .flatten()?;
    let dto = Json(VernacularDto::from(model.clone()));
    model
        .delete(&state.db)
        .await
        .map_err(|e| DcAppError::database(Some(e.into())))?;
    Ok(dto)
}
*/

pub fn get_v1_entries_router() -> Router<AppState> {
    Router::new()
        .route("/entries/{alias}", get(get_entry))
        .route("/entries", post(create_entry))
}
