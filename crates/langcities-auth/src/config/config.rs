use clap::Parser;
use figment::{
    Figment,
    providers::{Env, Format, Json, Serialized},
    util::map,
};
use langcities_common::merge::Merge;
use langcities_common_db::config::{DbConfig, PartialDbConfig};
use langcities_config::error::{LcConfigError, LcConfigErrorTrait};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialConfig {
    pub db: PartialDbConfig,
}

impl PartialConfig {
    pub fn new(db: impl Into<PartialDbConfig>) -> Self {
        let db = db.into();
        Self { db }
    }

    pub fn collect() -> Result<Self, LcConfigError> {
        let cli = PartialCli::parse();
        // db must be set to something otherwise the following error occurs:
        // Error: LcError { source: Some(Error { tag: Tag::Default, profile: Some(Profile(Uncased { string: "default" })), metadata: None, path: [], kind: MissingField("db"), prev: None }), kind: BadParse }
        let default = map!["db" => map!["url" => Option::<String>::None]];
        let mut config: Self = Figment::new()
            .merge(Serialized::from(default, "default"))
            .merge(Json::file("lcauth.json"))
            .merge(PartialDbConfig::modify_env_provider(Env::prefixed(
                "LCAUTH_",
            )))
            .extract()
            .map_err(|e| LcConfigError::bad_parse(Some(e.into())))?;
        config.merge_with(cli.into());
        Ok(config)
    }
}

impl Merge<PartialConfig> for PartialConfig {
    fn merge_with(&mut self, rhs: PartialConfig) {
        self.db.merge_with(rhs.db);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Parser)]
pub struct PartialCli {
    #[clap(flatten)]
    pub db: PartialDbConfig,
}

impl Into<PartialConfig> for PartialCli {
    fn into(self) -> PartialConfig {
        PartialConfig::new(self.db)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub db: DbConfig,
}

impl Config {
    pub fn new(db: impl Into<DbConfig>) -> Self {
        let db = db.into();
        Self { db }
    }

    pub fn from_partial(partial: impl Into<PartialConfig>) -> Result<Self, LcConfigError> {
        let partial = partial.into();
        let db = DbConfig::from_partial(partial.db)?;
        Ok(Self::new(db))
    }
}
