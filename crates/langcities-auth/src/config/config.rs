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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Parser)]
pub struct PartialAuthConfig {
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub disable_seeding: Option<bool>,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub seed_testing: Option<bool>,
}

impl Merge<PartialAuthConfig> for PartialAuthConfig {
    fn merge_with(&mut self, rhs: PartialAuthConfig) {
        self.disable_seeding.merge_with(rhs.disable_seeding);
        self.seed_testing.merge_with(rhs.seed_testing);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialConfig {
    pub auth: PartialAuthConfig,
    pub db: PartialDbConfig,
}

impl PartialConfig {
    pub fn new(auth: impl Into<PartialAuthConfig>, db: impl Into<PartialDbConfig>) -> Self {
        let (auth, db) = (auth.into(), db.into());
        Self { auth, db }
    }

    pub fn collect() -> Result<Self, LcConfigError> {
        let cli = PartialCli::parse();
        // db must be set to something otherwise the following error occurs:
        // Error: LcError { source: Some(Error { tag: Tag::Default, profile: Some(Profile(Uncased { string: "default" })), metadata: None, path: [], kind: MissingField("db"), prev: None }), kind: BadParse }
        let default = map![
            "auth" => map!["disable_seeding" => Option::<String>::None],
            "db" => map!["url" => Option::<String>::None]
        ];
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
        self.auth.merge_with(rhs.auth);
        self.db.merge_with(rhs.db);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Parser)]
pub struct PartialCli {
    #[clap(flatten)]
    pub auth: PartialAuthConfig,

    #[clap(flatten)]
    pub db: PartialDbConfig,
}

impl Into<PartialConfig> for PartialCli {
    fn into(self) -> PartialConfig {
        PartialConfig::new(self.auth, self.db)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthConfig {
    pub seed: bool,
    pub seed_testing: bool,
}

impl AuthConfig {
    pub fn from_partial(partial: PartialAuthConfig) -> Result<Self, LcConfigError> {
        Ok(Self {
            seed: !partial.disable_seeding.unwrap_or(false),
            seed_testing: partial.seed_testing.unwrap_or(false),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub auth: AuthConfig,
    pub db: DbConfig,
}

impl Config {
    pub fn new(auth: impl Into<AuthConfig>, db: impl Into<DbConfig>) -> Self {
        let (auth, db) = (auth.into(), db.into());
        Self { auth, db }
    }

    pub fn from_partial(partial: impl Into<PartialConfig>) -> Result<Self, LcConfigError> {
        let partial = partial.into();
        let db = DbConfig::from_partial(partial.db)?;
        let auth = AuthConfig::from_partial(partial.auth)?;
        Ok(Self::new(auth, db))
    }
}
