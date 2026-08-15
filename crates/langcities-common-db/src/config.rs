use std::time::Duration;

use figment::{providers::Env, value::Uncased};
use langcities_config::{
    datatype::{Milliseconds, ms_to_dur},
    error::{LcConfigError, LcConfigErrorTrait},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PartialDbConfig {
    pub url: Option<String>,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
    pub connect_timeout: Option<Milliseconds>,
    pub idle_timeout: Option<Option<Milliseconds>>,
    pub acquire_timeout: Option<Milliseconds>,
    pub max_lifetime: Option<Option<Milliseconds>>,
    pub sqlx_logging: Option<bool>,
    pub record_stmt_in_spans: Option<bool>,
    pub sqlx_logging_level: Option<log::LevelFilter>,
    pub sqlx_slow_statements_logging_level: Option<log::LevelFilter>,
    pub sqlx_slow_statements_logging_threshold: Option<Milliseconds>,
    pub sqlcipher_key: Option<String>,
    pub schema_search_path: Option<String>,
    pub application_name: Option<String>,
    pub statement_timeout: Option<Milliseconds>,
    pub test_before_acquire: Option<bool>,
    pub test_before_acquire_if_idle_for: Option<Milliseconds>,
    pub connect_lazy: Option<bool>,
}

impl PartialDbConfig {
    pub fn modify_env_provider(base: Env) -> Env {
        base.map(|k| {
            if k.starts_with("DB_") {
                Uncased::new(format!("db.{}", &k[3..]))
            } else if k.starts_with("DATABASE_") {
                Uncased::new(format!("db.{}", &k[9..]))
            } else {
                k.into()
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DbConfig {
    pub url: String,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
    pub connect_timeout: Option<Duration>,
    pub idle_timeout: Option<Option<Duration>>,
    pub acquire_timeout: Option<Duration>,
    pub max_lifetime: Option<Option<Duration>>,
    pub sqlx_logging: bool,
    pub record_stmt_in_spans: bool,
    pub sqlx_logging_level: log::LevelFilter,
    pub sqlx_slow_statements_logging_level: log::LevelFilter,
    pub sqlx_slow_statements_logging_threshold: Duration,
    pub sqlcipher_key: Option<String>,
    pub schema_search_path: Option<String>,
    pub application_name: Option<String>,
    pub statement_timeout: Option<Duration>,
    pub test_before_acquire: Option<bool>,
    pub test_before_acquire_if_idle_for: Option<Duration>,
    pub connect_lazy: bool,
}

impl DbConfig {
    pub fn from_partial(partial: impl Into<PartialDbConfig>) -> Result<Self, LcConfigError> {
        let partial = partial.into();
        let url = partial
            .url
            .ok_or_else(|| LcConfigError::missing_key("url"))?;
        let me = Self {
            url,
            max_connections: partial.max_connections,
            min_connections: partial.min_connections,
            connect_timeout: partial.connect_timeout.map(ms_to_dur),
            idle_timeout: partial.idle_timeout.map(|o| o.map(ms_to_dur)),
            acquire_timeout: partial.acquire_timeout.map(ms_to_dur),
            max_lifetime: partial.max_lifetime.map(|o| o.map(ms_to_dur)),
            sqlx_logging: partial.sqlx_logging.unwrap_or(true),
            record_stmt_in_spans: partial.record_stmt_in_spans.unwrap_or(true),
            sqlx_logging_level: partial.sqlx_logging_level.unwrap_or(log::LevelFilter::Info),
            sqlx_slow_statements_logging_level: partial
                .sqlx_slow_statements_logging_level
                .unwrap_or(log::LevelFilter::Off),
            sqlx_slow_statements_logging_threshold: partial
                .sqlx_slow_statements_logging_threshold
                .map(ms_to_dur)
                .unwrap_or(Duration::from_secs(1)),
            sqlcipher_key: partial.sqlcipher_key,
            schema_search_path: partial.schema_search_path,
            application_name: partial.application_name,
            statement_timeout: partial.statement_timeout.map(ms_to_dur),
            test_before_acquire: partial.test_before_acquire,
            test_before_acquire_if_idle_for: partial.test_before_acquire_if_idle_for.map(ms_to_dur),
            connect_lazy: partial.connect_lazy.unwrap_or(false),
        };
        Ok(me)
    }
}
