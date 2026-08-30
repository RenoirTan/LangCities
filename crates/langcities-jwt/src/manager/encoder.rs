use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::Serialize;

use crate::{
    config::{JwtConfig, KeyParams},
    error::{JwtError, JwtErrorTrait},
    payload::{Claims, claims::BaseClaims},
};

#[derive(Clone, Debug)]
pub struct JwtEncoder {
    encoding_key: EncodingKey,
    algorithm: Algorithm,
}

impl JwtEncoder {
    pub fn new<E, A>(encoding_key: E, algorithm: A) -> Self
    where
        E: Into<EncodingKey>,
        A: Into<Algorithm>,
    {
        let (encoding_key, algorithm) = (encoding_key.into(), algorithm.into());
        Self {
            encoding_key,
            algorithm,
        }
    }

    pub fn from_config(config: &JwtConfig) -> Result<Self, JwtError> {
        let key_config = config
            .key_config
            .as_ref()
            .ok_or_else(|| JwtError::bad_config(Some("key_config is None".into())))?;
        // TODO: make sure algorithm has a matching encoding key set
        // doesnt matter, if the config for another algorithm family is also set too
        let algorithm = key_config.algorithm;
        let encoding_key = match &key_config.params {
            KeyParams::Hmac(hmac) => EncodingKey::from_secret(&hmac.secret),
        };
        Ok(Self::new(encoding_key, algorithm))
    }

    pub fn get_header(&self) -> Header {
        Header::new(self.algorithm)
    }

    pub fn encode_claims<A>(
        &self,
        header: Header,
        claims: BaseClaims<A>,
    ) -> Result<String, JwtError>
    where
        A: Serialize,
    {
        let token = encode(&header, &claims, &self.encoding_key)
            .map_err(|e| JwtError::unencodeable(Some(e.into())))?;
        Ok(token)
    }

    pub fn encode_claims_enum(&self, header: Header, claims: Claims) -> Result<String, JwtError> {
        match claims {
            Claims::Generic(c) => self.encode_claims(header, c),
        }
    }
}
