use crate::entity::dc_users;
use crate::error::{DcAppError, DcAppErrorTrait};
use crate::state::AppState;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use langcities_jwt::payload::{Claims, ParseJwtClaims, ParsedClaims};
use sea_orm::ActiveValue::Set;
use sea_orm::{ConnectionTrait, EntityTrait, TransactionTrait, TryInsertResult};
use std::sync::Arc;

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
    .map_err(DcAppError::database)
    .map(|o| o.ok_or_else(|| DcAppError::other("could not insert new user")))
    .flatten()
}

pub async fn get_or_create_user<C: ConnectionTrait + TransactionTrait>(
    conn: &C,
    auth_user_id: i64,
) -> Result<dc_users::Model, DcAppError> {
    match dc_users::Entity::find_by_auth_user_id(auth_user_id)
        .one(conn)
        .await
    {
        Ok(Some(u)) => Ok(u),
        Ok(None) => create_user_if_not_exists(conn, auth_user_id).await,
        Err(e) => Err(DcAppError::database(e)),
    }
}

pub(crate) async fn resolve_current_user<C: ConnectionTrait + TransactionTrait>(
    conn: &C,
    claims: &Claims,
) -> Result<Option<dc_users::Model>, DcAppError> {
    let auth_user_id = claims.sub_to_id().map_err(DcAppError::unauthorized)?;
    match auth_user_id {
        Some(id) => get_or_create_user(conn, id).await.map(Some),
        None => Ok(None),
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
    if let Some(parsed_claims) = parsed_claims {
        if let ParsedClaims::Valid(claims) = &*parsed_claims {
            match resolve_current_user(&state.db, claims).await {
                Ok(Some(user)) => {
                    request.extensions_mut().insert(user);
                }
                Ok(None) => {}
                Err(error) => return error.into_response(),
            }
        }
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DcAppErrorKind;
    use sea_orm::DatabaseConnection;

    fn claims(sub: &str) -> Claims {
        Claims {
            jti: String::new(),
            aud: String::new(),
            exp: 0,
            iat: 0,
            iss: String::new(),
            nbf: 0,
            sub: sub.into(),
            scope: String::new(),
            resources: vec![],
        }
    }

    #[tokio::test]
    async fn malformed_subject_is_unauthorized() {
        let error = resolve_current_user(&DatabaseConnection::default(), &claims("not-an-id"))
            .await
            .unwrap_err();

        assert_eq!(error.kind, DcAppErrorKind::Unauthorized);
    }

    #[tokio::test]
    async fn absent_subject_keeps_request_anonymous() {
        let user = resolve_current_user(&DatabaseConnection::default(), &claims(""))
            .await
            .unwrap();

        assert!(user.is_none());
    }
}
