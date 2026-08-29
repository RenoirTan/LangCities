use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Serialize;

use crate::{
    error::{JwtError, JwtErrorTrait},
    payload::claims::BaseClaims,
};

#[derive(Clone, Debug)]
pub struct JwtEncoder {
    encoding_key: EncodingKey,
}

impl JwtEncoder {
    pub fn new<E>(encoding_key: E) -> Self
    where
        E: Into<EncodingKey>,
    {
        let encoding_key = encoding_key.into();
        Self { encoding_key }
    }

    pub fn get_header(&self) -> Header {
        Header::default()
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
}
