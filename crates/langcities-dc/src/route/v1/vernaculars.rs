use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use langcities_jwt::payload::Claims;
use langcities_lcdcdsl::component::AliasedResourceId;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use crate::{
    dto::vernaculars::{VernacularAliasDto, VernacularsCreateDto, VernacularsDto},
    entity::*,
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
    )
)]
#[axum::debug_handler]
pub async fn get_vernacular(
    Path(alias): Path<VernacularAliasDto>,
    State(state): State<AppState>,
) -> Result<Json<VernacularsDto>, DcAppError> {
    let id = match &alias.0 {
        AliasedResourceId::Id(id) => id.clone(),
        AliasedResourceId::Alias(_) => {
            return Err(DcAppError::other(Some("alias unsupported!".into())));
        }
    };
    println!("Id={:?}", id);
    let vernacular = vernaculars::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| DcAppError::database(Some(e.into())))
        .map(|o| {
            o.ok_or_else(|| DcAppError::not_found(Some(format!("could not find {}", alias).into())))
        })
        .flatten()?;
    Ok(Json(VernacularsDto::from(vernacular)))
}

#[utoipa::path(post, path = "/v1/vernaculars")]
#[axum::debug_handler]
pub async fn create_vernacular(
    State(state): State<AppState>,
    claims: Claims,
    Json(dto): Json<VernacularsCreateDto>,
) -> Result<Json<VernacularsDto>, DcAppError> {
    println!("{:#?}", dto);
    let owner_id = claims
        .sub_to_id()
        .map(|o| o.ok_or_else(|| DcAppError::unauthorized(Some("Login required".into()))))
        .map_err(|e| DcAppError::invalid_access_token(Some(e.into())))
        .flatten()?;
    let active_model = dto.to_active_model(owner_id);
    match active_model.insert(&state.db).await {
        Ok(model) => Ok(Json(model.into())),
        Err(DbErr::RecordNotInserted) => Err(DcAppError::bad_request(Some(
            DbErr::RecordNotInserted.into(),
        ))),
        Err(e) => Err(DcAppError::database(Some(e.into()))),
    }
}

pub fn get_v1_vernaculars_router() -> Router<AppState> {
    Router::new()
        .route("/vernaculars/{alias}", get(get_vernacular))
        .route("/vernaculars", post(create_vernacular))
}
