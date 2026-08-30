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

    pub fn get_sub(self) -> String {
        match self.user.id {
            Some(id) => id.to_string(),
            None => "".to_string(),
        }
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

    pub fn get_sub(self) -> String {
        match self {
            Self::Session(s) => s.get_sub(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Microservice {
    Generic,
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
        match self.microservice {
            Microservice::Generic => self.generate_generic_claims(generator),
        }
    }

    fn generate_generic_claims(self, generator: &ClaimsGenerator) -> Result<Claims, AuthAppError> {
        Ok(generator.generate_claims("generic", self.authorization.get_sub(), "", vec![]))
    }

    pub fn mint(self, state: &AppState) -> Result<AccessTokenResponseDto, AuthAppError> {
        let claims = self.generate_claims(&state.claims_generator)?;
        let header = state.jwt_encoder.get_header();
        let token = state
            .jwt_encoder
            .encode_claims(header, claims)
            .map_err(|e| AuthAppError::other(Some(e.into())))?;
        let expiry = state.claims_generator.expiry.num_seconds();
        Ok(AccessTokenResponseDto::new(token, "Bearer", expiry))
    }
}
