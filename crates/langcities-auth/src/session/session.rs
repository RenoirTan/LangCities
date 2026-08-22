use axum::extract::FromRequestParts;
use serde::{Deserialize, Serialize};
use tower_sessions::{Expiry, Session};

use crate::error::{AuthAppError, AuthAppErrorResponseDto, AuthAppErrorTrait};

#[derive(Clone, Debug)]
pub struct SessionUserWrapper {
    session: Session,
    user: SessionUserDto,
}

impl SessionUserWrapper {
    pub const USER_KEY: &'static str = "user";

    pub fn new<S, U>(session: S, user: U) -> Self
    where
        S: Into<Session>,
        U: Into<SessionUserDto>,
    {
        let (session, user) = (session.into(), user.into());
        Self { session, user }
    }

    pub fn get_user(&self) -> &SessionUserDto {
        &self.user
    }

    pub fn get_mut_user(&mut self) -> &mut SessionUserDto {
        &mut self.user
    }

    pub async fn update_user_session(&self) -> Result<(), AuthAppError> {
        self.session
            .insert(Self::USER_KEY, self.user.clone())
            .await
            .map_err(|e| AuthAppError::failed_session(Some(e.into())))
    }

    pub fn set_expiry(&self, expiry: Expiry) {
        self.session.set_expiry(Some(expiry));
    }
}

impl AsRef<SessionUserDto> for SessionUserWrapper {
    fn as_ref(&self) -> &SessionUserDto {
        self.get_user()
    }
}

impl AsMut<SessionUserDto> for SessionUserWrapper {
    fn as_mut(&mut self) -> &mut SessionUserDto {
        self.get_mut_user()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SessionUserDto {
    pub id: Option<i64>,
}

impl<S> FromRequestParts<S> for SessionUserWrapper
where
    S: Send + Sync,
{
    type Rejection = AuthAppErrorResponseDto;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|(_s, c)| AuthAppError::failed_session(Some(c.into())).to_response())?;
        let dto: SessionUserDto = session
            .get(Self::USER_KEY)
            .await
            .map_err(|e| AuthAppError::failed_session(Some(e.into())).to_response())?
            .unwrap_or_default();
        Ok(SessionUserWrapper::new(session, dto))
    }
}
