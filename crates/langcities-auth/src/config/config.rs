use std::collections::BTreeMap;

use clap::{Parser, ValueEnum};
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
use time::{Duration, OffsetDateTime};
use tower_sessions::{
    Expiry, SessionManagerLayer, SessionStore,
    cookie::Key,
    cookie::SameSite,
    service::{CookieController, PrivateCookie, SignedCookie},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum SameSiteArg {
    Strict,
    Lax,
    None,
}

impl SameSiteArg {
    pub fn to_same_site(&self) -> SameSite {
        match self {
            SameSiteArg::Strict => SameSite::Strict,
            SameSiteArg::Lax => SameSite::Lax,
            SameSiteArg::None => SameSite::None,
        }
    }
}

impl Into<SameSite> for SameSiteArg {
    fn into(self) -> SameSite {
        self.to_same_site()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Parser)]
pub struct PartialAuthConfig {
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub disable_seeding: Option<bool>,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub seed_testing: Option<bool>,

    #[arg(long)]
    pub session_cookie_name: Option<String>,

    #[arg(long)]
    pub session_cookie_secret: Option<String>,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub disable_session_cookie_http_only: Option<bool>,

    #[arg(long)]
    pub session_cookie_samesite: Option<SameSiteArg>,

    #[arg(long)]
    pub session_cookie_expiry: Option<Milliseconds>,

    #[arg(long)]
    pub session_cookie_reset_expiry_on_active: Option<bool>,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub disable_session_cookie_secure: Option<bool>,

    #[arg(long)]
    pub session_cookie_path: Option<String>,

    #[arg(long)]
    pub session_cookie_domain: Option<String>,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub session_cookie_always_save: Option<bool>,
}

impl Merge<PartialAuthConfig> for PartialAuthConfig {
    fn merge_with(&mut self, rhs: PartialAuthConfig) {
        self.disable_seeding.merge_with(rhs.disable_seeding);
        self.seed_testing.merge_with(rhs.seed_testing);
        self.session_cookie_name.merge_with(rhs.session_cookie_name);
        self.session_cookie_secret
            .merge_with(rhs.session_cookie_secret);
        self.disable_session_cookie_http_only
            .merge_with(rhs.disable_session_cookie_http_only);
        self.session_cookie_samesite
            .merge_with(rhs.session_cookie_samesite);
        self.session_cookie_expiry
            .merge_with(rhs.session_cookie_expiry);
        self.session_cookie_reset_expiry_on_active
            .merge_with(rhs.session_cookie_reset_expiry_on_active);
        self.disable_session_cookie_secure
            .merge_with(rhs.disable_session_cookie_secure);
        self.session_cookie_path.merge_with(rhs.session_cookie_path);
        self.session_cookie_domain
            .merge_with(rhs.session_cookie_domain);
        self.session_cookie_always_save
            .merge_with(rhs.session_cookie_always_save);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialConfig {
    pub auth: PartialAuthConfig,
    pub server: PartialServerConfig,
    pub db: PartialDbConfig,
    pub jwt: PartialJwtConfig,
}

impl PartialConfig {
    pub fn new<A, S, D, J>(auth: A, server: S, db: D, jwt: J) -> Self
    where
        A: Into<PartialAuthConfig>,
        S: Into<PartialServerConfig>,
        D: Into<PartialDbConfig>,
        J: Into<PartialJwtConfig>,
    {
        let (auth, server, db, jwt) = (auth.into(), server.into(), db.into(), jwt.into());
        Self {
            auth,
            server,
            db,
            jwt,
        }
    }

    pub fn collect() -> Result<Self, LcConfigError> {
        let cli = PartialCli::parse();
        // db must be set to something otherwise the following error occurs:
        // Error: LcError { source: Some(Error { tag: Tag::Default, profile: Some(Profile(Uncased { string: "default" })), metadata: None, path: [], kind: MissingField("db"), prev: None }), kind: BadParse }
        let default: BTreeMap<&str, BTreeMap<&str, Option<String>>> = map![
            "auth" => map![],
            "server" => map![],
            "db" => map![],
            "jwt" => map![],
        ];
        let mut config: Self = Figment::new()
            .merge(Serialized::from(default, "default"))
            .merge(Json::file("lcauth.json"))
            .merge(PartialDbConfig::modify_env_provider(Env::prefixed(
                "LCAUTH_",
            )))
            .merge(PartialServerConfig::modify_env_provider(Env::prefixed(
                "LCAUTH_",
            )))
            .merge(PartialJwtConfig::modify_env_provider(Env::prefixed(
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
        self.server.merge_with(rhs.server);
        self.db.merge_with(rhs.db);
        self.jwt.merge_with(rhs.jwt);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Parser)]
pub struct PartialCli {
    #[clap(flatten)]
    pub auth: PartialAuthConfig,

    #[clap(flatten)]
    pub server: PartialServerConfig,

    #[clap(flatten)]
    pub db: PartialDbConfig,

    #[clap(flatten)]
    pub jwt: PartialJwtConfig,
}

impl Into<PartialConfig> for PartialCli {
    fn into(self) -> PartialConfig {
        PartialConfig::new(self.auth, self.server, self.db, self.jwt)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthConfig {
    pub seed: bool,
    pub seed_testing: bool,
    pub session_cookie_name: Option<String>,
    pub session_cookie_secret: Option<String>,
    pub session_cookie_http_only: Option<bool>,
    pub session_cookie_samesite: Option<SameSiteArg>,
    pub session_cookie_expiry: Option<Milliseconds>,
    pub session_cookie_reset_expiry_on_active: Option<bool>,
    pub session_cookie_secure: Option<bool>,
    pub session_cookie_path: Option<String>,
    pub session_cookie_domain: Option<String>,
    pub session_cookie_always_save: Option<bool>,
}

impl AuthConfig {
    pub fn from_partial(partial: PartialAuthConfig) -> Result<Self, LcConfigError> {
        Ok(Self {
            seed: !partial.disable_seeding.unwrap_or(false),
            seed_testing: partial.seed_testing.unwrap_or(false),
            session_cookie_name: partial.session_cookie_name,
            session_cookie_secret: partial.session_cookie_secret,
            session_cookie_http_only: partial.disable_session_cookie_http_only.map(|b| !b),
            session_cookie_samesite: partial.session_cookie_samesite,
            session_cookie_expiry: partial.session_cookie_expiry,
            session_cookie_reset_expiry_on_active: partial.session_cookie_reset_expiry_on_active,
            session_cookie_secure: partial.disable_session_cookie_secure.map(|b| !b),
            session_cookie_path: partial.session_cookie_path,
            session_cookie_domain: partial.session_cookie_domain,
            session_cookie_always_save: partial.session_cookie_always_save,
        })
    }

    pub fn get_expiry(&self) -> Expiry {
        match self.session_cookie_expiry {
            Some(e) if e >= 1 => {
                let i = e.min(i64::MAX as u64) as i64;
                if self.session_cookie_reset_expiry_on_active.unwrap_or(false) {
                    Expiry::OnInactivity(Duration::milliseconds(i))
                } else {
                    Expiry::AtDateTime(OffsetDateTime::now_utc() + Duration::milliseconds(i))
                }
            }
            _ => Expiry::OnSessionEnd,
        }
    }

    pub fn configure_session_manager_layer_basics<S, C>(
        &self,
        mut layer: SessionManagerLayer<S, C>,
    ) -> SessionManagerLayer<S, C>
    where
        S: SessionStore,
        C: CookieController,
    {
        if let Some(name) = &self.session_cookie_name {
            layer = layer.with_name(name.to_string());
        }
        if let Some(http_only) = &self.session_cookie_http_only {
            layer = layer.with_http_only(*http_only);
        }
        if let Some(samesite) = &self.session_cookie_samesite {
            layer = layer.with_same_site((*samesite).into());
        }
        if let Some(secure) = &self.session_cookie_secure {
            layer = layer.with_secure(*secure);
        }
        if let Some(path) = &self.session_cookie_path {
            layer = layer.with_path(path.to_string());
        }
        if let Some(domain) = &self.session_cookie_domain {
            layer = layer.with_domain(domain.to_string());
        }
        if let Some(always_save) = &self.session_cookie_always_save {
            layer = layer.with_always_save(*always_save);
        }
        layer
    }

    pub fn session_cookie_secret_to_key(&self) -> Result<Key, LcConfigError> {
        match &self.session_cookie_secret {
            Some(secret) => Key::try_from(secret.as_bytes())
                .map_err(|e| LcConfigError::bad_parse(Some(e.into()))),
            None => Key::try_generate().ok_or_else(|| {
                LcConfigError::other(Some("Failed to generate a session cookie key".into()))
            }),
        }
    }

    pub fn configure_signed_cookies<S, C>(
        &self,
        layer: SessionManagerLayer<S, C>,
    ) -> Result<SessionManagerLayer<S, SignedCookie>, LcConfigError>
    where
        S: SessionStore,
        C: CookieController,
    {
        let key = self.session_cookie_secret_to_key()?;
        Ok(layer.with_signed(key))
    }

    pub fn configure_private_cookies<S, C>(
        &self,
        layer: SessionManagerLayer<S, C>,
    ) -> Result<SessionManagerLayer<S, PrivateCookie>, LcConfigError>
    where
        S: SessionStore,
        C: CookieController,
    {
        let key = self.session_cookie_secret_to_key()?;
        Ok(layer.with_private(key))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub auth: AuthConfig,
    pub server: ServerConfig,
    pub db: DbConfig,
    pub jwt: JwtConfig,
}

impl Config {
    pub fn new<A, S, D, J>(auth: A, server: S, db: D, jwt: J) -> Self
    where
        A: Into<AuthConfig>,
        S: Into<ServerConfig>,
        D: Into<DbConfig>,
        J: Into<JwtConfig>,
    {
        let (auth, server, db, jwt) = (auth.into(), server.into(), db.into(), jwt.into());
        Self {
            auth,
            server,
            db,
            jwt,
        }
    }

    pub fn from_partial(partial: impl Into<PartialConfig>) -> Result<Self, LcConfigError> {
        let partial = partial.into();
        let db = DbConfig::from_partial(partial.db)?;
        let server = ServerConfig::from_partial(partial.server, 8000)?;
        let auth = AuthConfig::from_partial(partial.auth)?;
        let jwt = JwtConfig::from_partial(partial.jwt, true)?;
        Ok(Self::new(auth, server, db, jwt))
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthConfig, PartialAuthConfig};

    fn auth_config_with_secret(secret: Option<String>) -> AuthConfig {
        AuthConfig::from_partial(PartialAuthConfig {
            session_cookie_secret: secret,
            ..Default::default()
        })
        .expect("auth config should be valid")
    }

    #[test]
    fn empty_session_cookie_secret_is_rejected() {
        let config = auth_config_with_secret(Some(String::new()));

        assert!(config.session_cookie_secret_to_key().is_err());
    }

    #[test]
    fn short_session_cookie_secret_is_rejected() {
        let config = auth_config_with_secret(Some("a".repeat(63)));

        assert!(config.session_cookie_secret_to_key().is_err());
    }

    #[test]
    fn sixty_four_byte_session_cookie_secret_is_accepted() {
        let config = auth_config_with_secret(Some("a".repeat(64)));

        assert!(config.session_cookie_secret_to_key().is_ok());
    }

    #[test]
    fn missing_session_cookie_secret_generates_a_key() {
        let config = auth_config_with_secret(None);

        assert!(config.session_cookie_secret_to_key().is_ok());
    }
}
