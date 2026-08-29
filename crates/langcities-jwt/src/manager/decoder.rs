use jsonwebtoken::{DecodingKey, TokenData, Validation, decode};
use serde::de::DeserializeOwned;

use crate::{
    error::{JwtError, JwtErrorTrait},
    payload::claims::BaseClaims,
};

#[derive(Clone, Debug)]
pub struct JwtDecoder {
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtDecoder {
    pub fn new<D, V>(decoding_key: D, validation: V) -> Self
    where
        D: Into<DecodingKey>,
        V: Into<Validation>,
    {
        let (decoding_key, validation) = (decoding_key.into(), validation.into());
        Self {
            decoding_key,
            validation,
        }
    }

    pub fn decode_token<A>(&self, token: &str) -> Result<TokenData<BaseClaims<A>>, JwtError>
    where
        A: DeserializeOwned,
    {
        let data = decode::<BaseClaims<A>>(token, &self.decoding_key, &self.validation)
            .map_err(|e| JwtError::undecodeable(Some(e.into())))?;
        Ok(data)
    }
}
