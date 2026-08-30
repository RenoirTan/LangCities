use clap::{Arg, Args, FromArgMatches, ValueEnum, value_parser};
use figment::{providers::Env, value::Uncased};
use jsonwebtoken::{Algorithm, AlgorithmFamily};
use langcities_common::merge::Merge;
use langcities_config::{
    datatype::{Milliseconds, ms_to_dur},
    error::{LcConfigError, LcConfigErrorTrait},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// TODO: add support for more algorithms
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[non_exhaustive]
pub enum JwtAlgorithm {
    HS256,
    HS384,
    HS512,
}

impl JwtAlgorithm {
    pub fn to_algorithm(self) -> Algorithm {
        match self {
            Self::HS256 => Algorithm::HS256,
            Self::HS384 => Algorithm::HS384,
            Self::HS512 => Algorithm::HS512,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PartialJwtConfig {
    pub hmac_secret: Option<String>,
    pub algorithm: Option<JwtAlgorithm>,
    pub expiry: Option<Milliseconds>,
    pub issuer: Option<String>,
}

impl PartialJwtConfig {
    pub fn modify_env_provider(base: Env) -> Env {
        base.map(|k| {
            if k.starts_with("JWT_") {
                Uncased::new(format!("jwt.{}", &k[4..]))
            } else {
                k.into()
            }
        })
    }

    pub fn augment_args_with_prefix(cmd: clap::Command, prefix: impl AsRef<str>) -> clap::Command {
        let prefix = prefix.as_ref();
        let p = |f: &str| format!("{}{}", prefix, f);
        cmd.arg(Arg::new("hmac_secret").long(p("hmac-secret")))
            .arg(
                Arg::new("algorithm")
                    .long(p("algorithm"))
                    .value_parser(value_parser!(JwtAlgorithm)),
            )
            .arg(
                Arg::new("expiry")
                    .long(p("expiry"))
                    .value_parser(value_parser!(Milliseconds)),
            )
            .arg(Arg::new("issuer").long(p("issuer")))
    }
}

impl Args for PartialJwtConfig {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        Self::augment_args_with_prefix(cmd, "jwt-")
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        Self::augment_args(cmd)
    }
}

impl FromArgMatches for PartialJwtConfig {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        let mut me = Self::default();
        me.update_from_arg_matches(matches)?;
        Ok(me)
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        matches
            .get_one::<String>("hmac_secret")
            .map(|s| self.hmac_secret.replace(s.clone()));
        matches
            .get_one::<JwtAlgorithm>("algorithm")
            .map(|a| self.algorithm.replace(*a));
        matches
            .get_one::<Milliseconds>("expiry")
            .map(|e| self.expiry.replace(*e));
        matches
            .get_one::<String>("issuer")
            .map(|i| self.issuer.replace(i.clone()));

        Ok(())
    }
}

impl Merge<PartialJwtConfig> for PartialJwtConfig {
    fn merge_with(&mut self, rhs: PartialJwtConfig) {
        self.hmac_secret.merge_with(rhs.hmac_secret);
        self.algorithm.merge_with(rhs.algorithm);
        self.expiry.merge_with(rhs.expiry);
        self.issuer.merge_with(rhs.issuer);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HmacConfig {
    pub secret: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyParams {
    Hmac(HmacConfig),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyConfig {
    pub params: KeyParams,
    pub algorithm: Algorithm,
}

impl KeyConfig {
    pub fn from_partial(partial: &PartialJwtConfig) -> Result<Option<Self>, LcConfigError> {
        let algorithm = match partial.algorithm {
            Some(a) => a.to_algorithm(),
            None => return Ok(None),
        };
        match algorithm.family() {
            AlgorithmFamily::Hmac => {
                if let Some(s) = &partial.hmac_secret {
                    let params = KeyParams::Hmac(HmacConfig {
                        secret: s.as_bytes().to_vec(),
                    });
                    Ok(Some(Self { params, algorithm }))
                } else {
                    Err(LcConfigError::missing_key("hmac-secret"))
                }
            }
            f => panic!("Unhandled algorithm families: {:?}", f),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JwtConfig {
    pub key_config: Option<KeyConfig>,
    pub expiry: Duration,
    pub issuer: String,
}

impl JwtConfig {
    pub fn from_partial(
        partial: impl Into<PartialJwtConfig>,
        require_key_config: bool,
    ) -> Result<Self, LcConfigError> {
        let partial = partial.into();
        let key_config = KeyConfig::from_partial(&partial)?;
        if require_key_config && let None = key_config {
            return Err(LcConfigError::missing_key("algorithm"));
        }
        let expiry = ms_to_dur(partial.expiry.unwrap_or(600000)); // 10 min
        let issuer = partial
            .issuer
            .unwrap_or_else(|| "http://localhost:8000".into());
        Ok(Self {
            key_config,
            expiry,
            issuer,
        })
    }
}
