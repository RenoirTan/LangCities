use clap::Parser;
use figment::{
    Figment,
    providers::{Env, Format, Json, Serialized},
    util::map,
    value::Uncased,
};
use langcities_cache::common::Expiry;
use langcities_common::merge::Merge;
use langcities_common_db::config::{DbConfig, PartialDbConfig};
use langcities_common_server::{
    auth_client::validate_base_url,
    config::{PartialServerConfig, ServerConfig},
};
use langcities_config::{
    datatype::Milliseconds,
    error::{LcConfigError, LcConfigErrorTrait},
};
use langcities_jwt::config::{JwtConfig, PartialJwtConfig};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Parser)]
pub struct PartialDcConfig {
    #[arg(long)]
    pub auth_base_url: Option<String>,

    #[arg(
        long,
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "true"
    )]
    pub seed_testing: Option<bool>,

    #[arg(long)]
    pub username_cache_ttl: Option<Milliseconds>,

    #[arg(long)]
    pub username_cache_max_capacity: Option<u64>,
}

impl PartialDcConfig {
    pub fn modify_env_provider(base: Env) -> Env {
        base.map(|k| {
            if k.starts_with("DC_") {
                Uncased::new(format!("dc.{}", &k[3..]))
            } else {
                k.into()
            }
        })
    }
}

impl Merge<PartialDcConfig> for PartialDcConfig {
    fn merge_with(&mut self, rhs: PartialDcConfig) {
        self.auth_base_url.merge_with(rhs.auth_base_url);
        self.seed_testing.merge_with(rhs.seed_testing);
        self.username_cache_ttl.merge_with(rhs.username_cache_ttl);
        self.username_cache_max_capacity
            .merge_with(rhs.username_cache_max_capacity);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialConfig {
    pub dc: PartialDcConfig,
    pub server: PartialServerConfig,
    pub db: PartialDbConfig,
    pub jwt: PartialJwtConfig,
}

impl PartialConfig {
    pub fn new<C, S, D, J>(dc: C, server: S, db: D, jwt: J) -> Self
    where
        C: Into<PartialDcConfig>,
        S: Into<PartialServerConfig>,
        D: Into<PartialDbConfig>,
        J: Into<PartialJwtConfig>,
    {
        let (dc, server, db, jwt) = (dc.into(), server.into(), db.into(), jwt.into());
        Self {
            dc,
            server,
            db,
            jwt,
        }
    }

    pub fn collect() -> Result<Self, LcConfigError> {
        let cli = PartialCli::parse();
        // dc/server/db/jwt must resolve to a section or extraction fails with a MissingField error
        // before the per-provider values are merged in.
        let default: BTreeMap<&str, BTreeMap<&str, Option<String>>> = map![
            "dc" => map![],
            "server" => map![],
            "db" => map![],
            "jwt" => map![],
        ];
        let env_provider = PartialDcConfig::modify_env_provider(
            PartialDbConfig::modify_env_provider(PartialServerConfig::modify_env_provider(
                PartialJwtConfig::modify_env_provider(Env::prefixed("LCDC_")),
            )),
        );
        let mut config: Self = Figment::new()
            .merge(Serialized::from(default, "default"))
            .merge(Json::file("lcdc.json"))
            .merge(env_provider)
            .extract()
            .map_err(|e| LcConfigError::bad_parse(Some(e.into())))?;
        config.merge_with(cli.into());
        Ok(config)
    }
}

impl Merge<PartialConfig> for PartialConfig {
    fn merge_with(&mut self, rhs: PartialConfig) {
        self.dc.merge_with(rhs.dc);
        self.server.merge_with(rhs.server);
        self.db.merge_with(rhs.db);
        self.jwt.merge_with(rhs.jwt);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Parser)]
pub struct PartialCli {
    #[clap(flatten)]
    pub dc: PartialDcConfig,

    #[clap(flatten)]
    pub server: PartialServerConfig,

    #[clap(flatten)]
    pub db: PartialDbConfig,

    #[clap(flatten)]
    pub jwt: PartialJwtConfig,
}

impl Into<PartialConfig> for PartialCli {
    fn into(self) -> PartialConfig {
        PartialConfig::new(self.dc, self.server, self.db, self.jwt)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DcConfig {
    pub auth_base_url: url::Url,
    pub seed_testing: bool,
    pub username_cache_expiry: Expiry,
    pub username_cache_max_capacity: u64,
}

impl DcConfig {
    pub fn from_partial(partial: PartialDcConfig) -> Result<Self, LcConfigError> {
        let auth_base_url = url::Url::parse(
            partial
                .auth_base_url
                .as_deref()
                .unwrap_or("http://localhost:8000"),
        )
        .map_err(|e| LcConfigError::bad_parse(Some(e.into())))?;
        validate_base_url(&auth_base_url)?;
        Ok(Self {
            auth_base_url,
            seed_testing: partial.seed_testing.unwrap_or(false),
            username_cache_expiry: match partial.username_cache_ttl {
                None => Expiry::None,
                Some(ms) => Expiry::Ttl(ms),
            },
            username_cache_max_capacity: partial.username_cache_max_capacity.unwrap_or(10000),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub dc: DcConfig,
    pub server: ServerConfig,
    pub db: DbConfig,
    pub jwt: JwtConfig,
}

impl Config {
    pub fn new<C, S, D, J>(dc: C, server: S, db: D, jwt: J) -> Self
    where
        C: Into<DcConfig>,
        S: Into<ServerConfig>,
        D: Into<DbConfig>,
        J: Into<JwtConfig>,
    {
        let (dc, server, db, jwt) = (dc.into(), server.into(), db.into(), jwt.into());
        Self {
            dc,
            server,
            db,
            jwt,
        }
    }

    pub fn from_partial(partial: impl Into<PartialConfig>) -> Result<Self, LcConfigError> {
        let partial = partial.into();
        let dc = DcConfig::from_partial(partial.dc)?;
        let server = ServerConfig::from_partial(partial.server, 8032)?;
        let db = DbConfig::from_partial(partial.db)?;
        let jwt = JwtConfig::from_partial(partial.jwt, true)?;
        Ok(Self::new(dc, server, db, jwt))
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn auth_base_url_is_parsed_and_validated() {
        for value in ["not a url", "/relative", "ftp://localhost"] {
            assert!(
                DcConfig::from_partial(PartialDcConfig {
                    auth_base_url: Some(value.into()),
                    ..Default::default()
                })
                .is_err(),
                "accepted {value}"
            );
        }
        for value in ["http://localhost:8000", "https://auth.example.com/"] {
            let config = DcConfig::from_partial(PartialDcConfig {
                auth_base_url: Some(value.into()),
                ..Default::default()
            })
            .unwrap();
            assert_eq!(config.auth_base_url, url::Url::parse(value).unwrap());
        }
        let config = DcConfig::from_partial(PartialDcConfig::default()).unwrap();
        assert_eq!(config.auth_base_url.as_str(), "http://localhost:8000/");
    }

    #[test]
    fn auth_url_precedence_is_cli_then_env_then_json() {
        figment::Jail::expect_with(|jail| {
            jail.set_env("LCDC_DC_AUTH_BASE_URL", "http://env:8000");
            jail.set_env("LCDC_DC_USERNAME_CACHE_TTL", "30000");
            let mut config: PartialDcConfig = Figment::new()
                .merge(Json::string(
                    r#"{"dc":{"auth_base_url":"http://json:8000"}}"#,
                ))
                .merge(PartialDcConfig::modify_env_provider(Env::prefixed("LCDC_")))
                .extract_inner("dc")?;
            assert_eq!(config.auth_base_url.as_deref(), Some("http://env:8000"));
            assert_eq!(config.username_cache_ttl, Some(30000));

            // Omitted CLI values must preserve lower-priority settings.
            config.merge_with(PartialDcConfig::try_parse_from(["test"]).unwrap());
            assert_eq!(config.auth_base_url.as_deref(), Some("http://env:8000"));
            config.merge_with(
                PartialDcConfig::try_parse_from(["test", "--auth-base-url", "http://cli:8000"])
                    .unwrap(),
            );
            assert_eq!(config.auth_base_url.as_deref(), Some("http://cli:8000"));
            Ok(())
        });
    }

    #[test]
    fn omitted_seed_testing_is_none() {
        let config = PartialDcConfig::try_parse_from(["test"]).unwrap();

        assert_eq!(config.seed_testing, None);
    }

    #[test]
    fn bare_seed_testing_is_true() {
        let config = PartialDcConfig::try_parse_from(["test", "--seed-testing"]).unwrap();

        assert_eq!(config.seed_testing, Some(true));
    }

    #[test]
    fn explicit_seed_testing_false_is_false() {
        let config = PartialDcConfig::try_parse_from(["test", "--seed-testing=false"]).unwrap();

        assert_eq!(config.seed_testing, Some(false));
    }

    #[test]
    fn username_cache_options_are_parsed() {
        let config = PartialDcConfig::try_parse_from([
            "test",
            "--username-cache-ttl",
            "30000",
            "--username-cache-max-capacity",
            "250",
        ])
        .unwrap();

        assert_eq!(config.username_cache_ttl, Some(30000));
        assert_eq!(config.username_cache_max_capacity, Some(250));
    }
}
