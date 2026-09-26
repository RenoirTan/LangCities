use crate::entity::dc_users;
use crate::error::{DcAppError, DcAppErrorTrait};
use crate::repo::user::UserRepo;
use crate::state::AppState;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use langcities_jwt::payload::{Claims, ParseJwtClaims, ParsedClaims};
use sea_orm::{ConnectionTrait, TransactionTrait};
use std::sync::Arc;

pub(crate) async fn resolve_current_user<C: ConnectionTrait + TransactionTrait>(
    conn: &C,
    claims: &Claims,
) -> Result<Option<dc_users::Model>, DcAppError> {
    let auth_user_id = claims.sub_to_id().map_err(DcAppError::unauthorized)?;
    match auth_user_id {
        Some(id) => UserRepo.get_or_create_user(conn, id).await.map(Some),
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
