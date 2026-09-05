use std::sync::Arc;

use crate::entity::dc_users;
use crate::error::{DcAppError, DcAppErrorTrait};
use crate::state::AppState;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use langcities_jwt::payload::{ParseJwtClaims, ParsedClaims};
use sea_orm::ActiveValue::Set;
use sea_orm::{ConnectionTrait, EntityTrait, TransactionTrait, TryInsertResult};

pub async fn create_user_if_not_exists<C: TransactionTrait>(
    conn: &C,
    id: i64,
) -> Result<dc_users::Model, DcAppError> {
    let user = dc_users::ActiveModel {
        auth_user_id: Set(id),
        ..Default::default()
    };
    conn.transaction(|txn| {
        Box::pin(async move {
            match dc_users::Entity::insert(user)
                .on_conflict_do_nothing_on([dc_users::Column::AuthUserId])
                .exec_with_returning(txn)
                .await
            {
                Ok(TryInsertResult::Empty | TryInsertResult::Conflicted) => {
                    dc_users::Entity::find_by_auth_user_id(id).one(txn).await
                }
                Ok(TryInsertResult::Inserted(m)) => Ok(Some(m)),
                Err(e) => Err(e),
            }
        })
    })
    .await
    .map_err(|e| DcAppError::database(Some(e.into())))
    .map(|o| o.ok_or_else(|| DcAppError::database(Some("Could not insert new user".into()))))
    .flatten()
}

pub async fn get_or_create_user<C: ConnectionTrait + TransactionTrait>(
    conn: &C,
    id: i64,
) -> Result<dc_users::Model, DcAppError> {
    match dc_users::Entity::find_by_auth_user_id(id).one(conn).await {
        Ok(Some(u)) => Ok(u),
        Ok(None) => create_user_if_not_exists(conn, id).await,
        Err(e) => Err(DcAppError::database(Some(e.into()))),
    }
}

pub async fn extract_current_user(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let parsed_claims = request
        .extensions()
        .get::<Arc<ParsedClaims<<AppState as ParseJwtClaims>::Error>>>()
        .cloned();
    // println!("{:?}", parsed_claims);
    if let Some(parsed_claims) = parsed_claims {
        if let ParsedClaims::Valid(claims) = &*parsed_claims {
            if let Ok(Some(id)) = claims.sub_to_id() {
                match get_or_create_user(&state.db, id).await {
                    Ok(user) => {
                        request.extensions_mut().insert(user);
                    }
                    Err(error) => return error.into_response(),
                }
            }
        }
    }
    next.run(request).await
}
