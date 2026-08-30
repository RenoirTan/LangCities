pub mod decoder;
pub mod encoder;

pub use self::decoder::*;
pub use self::encoder::*;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use jsonwebtoken::Algorithm;

    use crate::{
        config::{HmacConfig, JwtConfig, KeyConfig, KeyParams},
        payload::ClaimsGenerator,
    };

    use super::*;

    #[test]
    fn codec_jwt_0() {
        let config = JwtConfig {
            key_config: Some(KeyConfig {
                algorithm: Algorithm::HS256,
                params: KeyParams::Hmac(HmacConfig {
                    secret: "secret".as_bytes().to_vec(),
                }),
            }),
            expiry: Duration::from_secs(600),
            issuer: "http://localhost:8000".to_string(),
        };
        let generator = ClaimsGenerator::from_config(&config).unwrap();
        let encoder = JwtEncoder::from_config(&config).unwrap();
        let decoder = JwtDecoder::from_config(&config, &["generic"]).unwrap();

        let claims = generator.generate_generic_claims("generic", "123");
        let token = encoder.encode_claims(encoder.get_header(), claims).unwrap();
        let decoded = decoder.decode_token::<()>(&token).unwrap();
        println!("{:#?}", decoded);
    }
}
