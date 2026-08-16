use std::time::Duration;

use clap::{Arg, ArgAction::SetTrue, Args, FromArgMatches, value_parser};
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

    pub fn augment_args_with_prefix(cmd: clap::Command, prefix: impl AsRef<str>) -> clap::Command {
        let prefix = prefix.as_ref();
        let p = |f: &str| format!("{}{}", prefix, f);

        cmd.arg(
            Arg::new("url")
                .long(p("url"))
                .help("Connection string for database"),
        )
        .arg(
            Arg::new("max_connections")
                .long(p("max-connections"))
                .value_parser(value_parser!(u32)),
        )
        .arg(
            Arg::new("min_connections")
                .long(p("min-connections"))
                .value_parser(value_parser!(u32)),
        )
        .arg(
            Arg::new("connect_timeout")
                .long(p("connect-timeout"))
                .value_parser(value_parser!(Milliseconds)),
        )
        .arg(
            Arg::new("idle_timeout")
                .long(p("idle-timeout"))
                .value_parser(value_parser!(Milliseconds)),
        )
        .arg(
            Arg::new("acquire_timeout")
                .long(p("acquire-timeout"))
                .value_parser(value_parser!(Milliseconds)),
        )
        .arg(
            Arg::new("max_lifetime")
                .long(p("max-lifetime"))
                .value_parser(value_parser!(Milliseconds)),
        )
        .arg(
            Arg::new("disable_sqlx_logging")
                .long(p("disable-sqlx-logging"))
                .action(SetTrue),
        )
        .arg(
            Arg::new("disable_record_stmt_in_spans")
                .long(p("disable-record-stmt-in-spans"))
                .action(SetTrue),
        )
        .arg(Arg::new("sqlx_logging_level").long(p("sqlx-logging-level")))
        .arg(
            Arg::new("sqlx_slow_statements_logging_level")
                .long(p("sqlx-slow-statements-logging-level")),
        )
        .arg(
            Arg::new("sqlx_slow_statements_logging_threshold")
                .long(p("sqlx-slow-statements-logging-threshold"))
                .value_parser(value_parser!(Milliseconds)),
        )
        .arg(Arg::new("sqlcipher_key").long(p("sqlcipher-key")))
        .arg(Arg::new("schema_search_path").long(p("schema_search_path")))
        .arg(Arg::new("application_name").long(p("application-name")))
        .arg(
            Arg::new("statement_timeout")
                .long(p("statement-timeout"))
                .value_parser(value_parser!(Milliseconds)),
        )
        .arg(
            Arg::new("test_before_acquire")
                .long(p("test-before-acquire"))
                .action(SetTrue),
        )
        .arg(
            Arg::new("test_before_acquire_if_idle_for")
                .long(p("test-before-acquire-if-idle-for"))
                .value_parser(value_parser!(Milliseconds)),
        )
        .arg(
            Arg::new("connect_lazy")
                .long(p("connect-lazy"))
                .action(SetTrue),
        )
    }
}

impl Args for PartialDbConfig {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        Self::augment_args_with_prefix(cmd, "db-")
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        Self::augment_args(cmd)
    }
}

impl FromArgMatches for PartialDbConfig {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        let mut me = Self::default();
        me.update_from_arg_matches(matches)?;
        Ok(me)
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        matches
            .get_one::<String>("url")
            .map(|u| self.url.replace(u.clone()));
        matches
            .get_one::<u32>("max_connections")
            .map(|c| self.max_connections.replace(*c));
        matches
            .get_one::<u32>("min_connections")
            .map(|c| self.min_connections.replace(*c));
        matches
            .get_one::<Milliseconds>("connect_timeout")
            .map(|t| self.connect_timeout.replace(*t));
        matches.get_one::<Milliseconds>("idle_timeout").map(|t| {
            self.idle_timeout.replace(match *t {
                0 => None,
                t => Some(t),
            })
        });
        matches
            .get_one::<Milliseconds>("acquire_timeout")
            .map(|t| self.acquire_timeout.replace(*t));
        matches.get_one::<Milliseconds>("max_lifetime").map(|t| {
            self.max_lifetime.replace(match *t {
                0 => None,
                t => Some(t),
            })
        });
        if matches.get_flag("disable_sqlx_logging") {
            self.sqlx_logging.replace(false);
        }
        if matches.get_flag("disable_record_stmt_in_spans") {
            self.record_stmt_in_spans.replace(false);
        }
        matches
            .get_one::<log::LevelFilter>("sqlx_logging_level")
            .map(|l| self.sqlx_logging_level.replace(l.clone()));
        matches
            .get_one::<log::LevelFilter>("sqlx_slow_statements_logging_level")
            .map(|l| self.sqlx_slow_statements_logging_level.replace(l.clone()));
        matches
            .get_one::<Milliseconds>("sqlx_slow_statements_logging_threshold")
            .map(|t| self.sqlx_slow_statements_logging_threshold.replace(*t));
        matches
            .get_one::<String>("sqlcipher_key")
            .map(|k| self.sqlcipher_key.replace(k.clone()));
        matches
            .get_one::<String>("schema_search_path")
            .map(|p| self.schema_search_path.replace(p.clone()));
        matches
            .get_one::<String>("application_name")
            .map(|n| self.application_name.replace(n.clone()));
        matches
            .get_one::<Milliseconds>("statement_timeout")
            .map(|t| self.statement_timeout.replace(*t));
        if matches.get_flag("test_before_acquire") {
            self.test_before_acquire.replace(true);
        }
        matches
            .get_one::<Milliseconds>("test_before_acquire_if_idle_for")
            .map(|t| self.test_before_acquire_if_idle_for.replace(*t));
        if matches.get_flag("connect_lazy") {
            self.connect_lazy.replace(true);
        }
        Ok(())
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
