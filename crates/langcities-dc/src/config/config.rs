use chrono::Duration;
use clap::Parser;
use figment::{
    Figment,
    providers::{Env, Format, Json, Serialized},
    util::map,
};
use langcities_common::merge::Merge;
use langcities_common_db::config::{DbConfig, PartialDbConfig};
use langcities_common_server::config::{PartialServerConfig, ServerConfig};
use langcities_config::{
    datatype::Milliseconds,
    error::{LcConfigError, LcConfigErrorTrait},
};
use langcities_jwt::config::{JwtConfig, PartialJwtConfig};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Parser)]
pub struct PartialDcConfig {
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

impl Merge<PartialDcConfig> for PartialDcConfig {
    fn merge_with(&mut self, rhs: PartialDcConfig) {
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
        let env_provider =
            PartialDbConfig::modify_env_provider(PartialServerConfig::modify_env_provider(
                PartialJwtConfig::modify_env_provider(Env::prefixed("LCDC_")),
            ));
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
    pub seed_testing: bool,
    pub username_cache_ttl: Duration,
    pub username_cache_max_capacity: u64,
}

impl DcConfig {
    pub fn from_partial(partial: PartialDcConfig) -> Self {
        Self {
            seed_testing: partial.seed_testing.unwrap_or(false),
            username_cache_ttl: Duration::milliseconds(
                partial.username_cache_ttl.unwrap_or(600000) as i64, // 10 minutes
            ),
            username_cache_max_capacity: partial.username_cache_max_capacity.unwrap_or(10000),
        }
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
        let dc = DcConfig::from_partial(partial.dc);
        let server = ServerConfig::from_partial(partial.server, 8032)?;
        let db = DbConfig::from_partial(partial.db)?;
        let jwt = JwtConfig::from_partial(partial.jwt, true)?;
        Ok(Self::new(dc, server, db, jwt))
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::PartialDcConfig;

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
