use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use sea_orm::{ActiveModelTrait, DbErr};

use crate::{
    dto::vernaculars::{VernacularAliasDto, VernacularsCreateDto, VernacularsDto},
    entity::dc_users,
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
    alias
        .resolve(&state.db)
        .await
        .map(|o| {
            o.map(|m| Json(m.into())).ok_or_else(|| {
                DcAppError::not_found(Some(format!("vernacular {} not found", alias).into()))
            })
        })
        .flatten()
}

#[utoipa::path(post, path = "/v1/vernaculars")]
#[axum::debug_handler]
pub async fn create_vernacular(
    State(state): State<AppState>,
    user: dc_users::Model,
    Json(dto): Json<VernacularsCreateDto>,
) -> Result<Json<VernacularsDto>, DcAppError> {
    println!("{:#?}", dto);
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

pub fn get_v1_vernaculars_router() -> Router<AppState> {
    Router::new()
        .route("/vernaculars/{alias}", get(get_vernacular))
        .route("/vernaculars", post(create_vernacular))
}
