use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, patch, post},
};
use langcities_common_server::dto::request::RequestContext;
use sea_orm::{ActiveModelTrait, DbErr, ModelTrait};

use crate::{
    dto::vernaculars::{
        CreateVernacularDto, UpdateVernacularDto, VernacularAccessDto, VernacularAliasDto,
        VernacularDto,
    },
    entity::{dc_users, vernaculars},
    error::{DcAppError, DcAppErrorTrait},
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
    VernacularAccessDto::read(alias.clone(), request_context)
        .resolve(&state.db, &state)
        .await
        .map(|o| {
            o.map(|m| Json(m.into()))
                .ok_or_else(|| DcAppError::not_found(format!("vernacular {} not found", alias)))
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
    Json(dto): Json<CreateVernacularDto>,
) -> Result<Json<VernacularDto>, DcAppError> {
    let owner_id = user.id;
    let active_model = dto.to_active_model(owner_id);
    match active_model.insert(&state.db).await {
        Ok(model) => Ok(Json(model.into())),
        Err(DbErr::RecordNotInserted) => Err(DcAppError::conflict(DbErr::RecordNotInserted)),
        Err(e) => Err(DcAppError::database(e)),
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
        .map(|o| o.ok_or_else(|| DcAppError::not_found(format!("vernacular {} not found", alias))))
        .flatten()?;
    let dto = Json(VernacularDto::from(model.clone()));
    model
        .delete(&state.db)
        .await
        .map_err(DcAppError::database)?;
    Ok(dto)
}

pub fn get_v1_vernaculars_router() -> Router<AppState> {
    Router::new()
        .route("/vernaculars/{alias}", get(get_vernacular))
        .route("/vernaculars", post(create_vernacular))
        .route("/vernaculars/{alias}", patch(update_vernacular))
        .route("/vernaculars/{alias}", delete(delete_vernacular))
}
