use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use langcities_lcdcdsl::component::AliasedResourceId;
use sea_orm::EntityTrait;

use crate::{
    dto::vernaculars::{VernacularAliasDto, VernacularsDto},
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

pub fn get_v1_vernaculars_router() -> Router<AppState> {
    Router::new().route("/vernaculars/{alias}", get(get_vernacular))
}
