use chrono::{Duration, Utc};

use crate::payload::BaseClaims;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimsGenerator {
    pub expiry: Duration,
    pub issuer: String,
    pub jti_counter: usize,
}

impl ClaimsGenerator {
    pub fn new<E, I, J>(expiry: E, issuer: I, jti_counter: J) -> Self
    where
        E: Into<Duration>,
        I: Into<String>,
        J: Into<usize>,
    {
        let (expiry, issuer, jti_counter) = (expiry.into(), issuer.into(), jti_counter.into());
        Self {
            expiry,
            issuer,
            jti_counter,
        }
    }

    pub fn generate_claims<A>(
        &mut self,
        audience: impl Into<String>,
        subject: impl Into<String>,
        additional: A,
    ) -> BaseClaims<A> {
        let jti = self.jti_counter.to_string();
        self.jti_counter += 1;
        let aud = audience.into();
        let iat = Utc::now().timestamp();
        let exp = iat + self.expiry.num_seconds();
        let iss = self.issuer.clone();
        let sub = subject.into();
        BaseClaims {
            jti,
            aud,
            exp,
            iat,
            nbf: iat,
            iss,
            sub,
            add: additional,
        }
    }
}
