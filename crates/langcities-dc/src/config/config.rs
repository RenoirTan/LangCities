use std::collections::BTreeMap;

use clap::Parser;
use figment::{
    Figment,
    providers::{Env, Format, Json, Serialized},
    util::map,
};
use langcities_common::merge::Merge;
use langcities_common_db::config::{DbConfig, PartialDbConfig};
use langcities_common_server::config::{PartialServerConfig, ServerConfig};
use langcities_config::error::{LcConfigError, LcConfigErrorTrait};
use langcities_jwt::config::{JwtConfig, PartialJwtConfig};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Parser)]
pub struct PartialDcConfig {
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub seed_testing: Option<bool>,
}

impl Merge<PartialDcConfig> for PartialDcConfig {
    fn merge_with(&mut self, rhs: PartialDcConfig) {
        self.seed_testing.merge_with(rhs.seed_testing);
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
        let mut config: Self = Figment::new()
            .merge(Serialized::from(default, "default"))
            .merge(Json::file("lcdc.json"))
            .merge(PartialDbConfig::modify_env_provider(Env::prefixed("LCDC_")))
            .merge(PartialServerConfig::modify_env_provider(Env::prefixed(
                "LCDC_",
            )))
            .merge(PartialJwtConfig::modify_env_provider(Env::prefixed(
                "LCDC_",
            )))
            .extract()
            .map_err(|e| LcConfigError::bad_parse(Some(e.into())))?;
        println!("{:#?}", config);
        println!("{:#?}", cli);
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
}

impl DcConfig {
    pub fn from_partial(partial: PartialDcConfig) -> Self {
        Self {
            seed_testing: partial.seed_testing.unwrap_or(false),
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
