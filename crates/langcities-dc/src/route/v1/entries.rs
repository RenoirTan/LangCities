use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use langcities_common_server::dto::request::RequestContext;

use crate::{
    dto::entries::{EntryAccessDto, EntryAliasDto, EntryDto},
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

/*
#[utoipa::path(
    post,
    path = "/v1/vernaculars",
    request_body = CreateVernacularDto,
    responses(
        (status = 200, body = VernacularDto, description = "new vernacular details")
    )
)]
#[axum::debug_handler]
pub async fn create_vernacular(
    State(state): State<AppState>,
    user: dc_users::Model,
    Json(dto): Json<CreateVernacularDto>,
) -> Result<Json<VernacularDto>, DcAppError> {
    let owner_id = user.id;
    let active_model = dto.to_active_model(owner_id);
    match active_model.insert(&state.db).await {
        Ok(model) => Ok(Json(model.into())),
        Err(DbErr::RecordNotInserted) => Err(DcAppError::bad_request(Some(
            DbErr::RecordNotInserted.into(),
        ))),
        Err(e) => Err(DcAppError::database(Some(e.into()))),
    }
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
    Router::new().route("/entries/{alias}", get(get_entry))
}
