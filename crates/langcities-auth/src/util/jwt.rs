pub use langcities_jwt::microservice::Microservice;
use langcities_jwt::payload::{Claims, ClaimsGenerator};

use crate::{
    dto::token::AccessTokenResponseDto,
    error::{AuthAppError, AuthAppErrorTrait},
    session::SessionUserDto,
    state::AppState,
};

#[derive(Clone, Debug)]
pub struct SessionAuthorization {
    pub user: SessionUserDto,
}

impl SessionAuthorization {
    pub fn new<U>(user: U) -> Self
    where
        U: Into<SessionUserDto>,
    {
        let user = user.into();
        Self { user }
    }

    pub fn get_sub(self) -> Result<String, AuthAppError> {
        self.user
            .id
            .map(|id| id.to_string())
            .ok_or_else(|| AuthAppError::unauthorized("not logged in"))
    }
}

#[derive(Clone, Debug)]
pub enum Authorization {
    Session(SessionAuthorization),
}

impl Authorization {
    pub fn session<U>(user: U) -> Self
    where
        U: Into<SessionUserDto>,
    {
        Self::Session(SessionAuthorization::new(user))
    }

    pub fn get_sub(self) -> Result<String, AuthAppError> {
        match self {
            Self::Session(s) => s.get_sub(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Access {
    pub authorization: Authorization,
    pub microservice: Microservice,
}

impl Access {
    pub fn new<A, M>(authorization: A, microservice: M) -> Self
    where
        A: Into<Authorization>,
        M: Into<Microservice>,
    {
        let (authorization, microservice) = (authorization.into(), microservice.into());
        Self {
            authorization,
            microservice,
        }
    }

    pub fn generate_claims(self, generator: &ClaimsGenerator) -> Result<Claims, AuthAppError> {
        Ok(generator.generate_claims(
            self.microservice.first_allowed_audience(),
            self.authorization.get_sub()?,
            "all",
            vec![],
        ))
    }

    pub fn mint(self, state: &AppState) -> Result<AccessTokenResponseDto, AuthAppError> {
        let claims = self.generate_claims(&state.claims_generator)?;
        let header = state.jwt_encoder.get_header();
        let token = state
            .jwt_encoder
            .encode_claims(header, claims)
            .map_err(AuthAppError::other)?;
        let expiry = state.claims_generator.expiry.num_seconds();
        Ok(AccessTokenResponseDto::new(token, "Bearer", expiry))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AuthAppErrorKind;

    #[test]
    fn session_authorization_requires_a_user_id() {
        let error = SessionAuthorization::new(SessionUserDto::default())
            .get_sub()
            .unwrap_err();

        assert_eq!(error.kind, AuthAppErrorKind::Unauthorized);
    }
}
