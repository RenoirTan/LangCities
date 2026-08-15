use figment::{
    Figment,
    providers::{Env, Format, Json},
};
use langcities_common_db::config::{DbConfig, PartialDbConfig};
use langcities_config::error::{LcConfigError, LcConfigErrorTrait};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialConfig {
    #[serde(flatten)]
    pub db: PartialDbConfig,
}

impl PartialConfig {
    pub fn new(db: impl Into<PartialDbConfig>) -> Self {
        let db = db.into();
        Self { db }
    }

    pub fn collect() -> Result<Self, LcConfigError> {
        let config: Self = Figment::new()
            .merge(Json::file("lcauth.json"))
            .merge(Env::prefixed("LCAUTH_"))
            .extract()
            .map_err(|e| LcConfigError::bad_parse(Some(e.into())))?;
        Ok(config)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(flatten)]
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
