use std::{
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};

use clap::{Arg, Args, FromArgMatches, value_parser};
use figment::{providers::Env, value::Uncased};
use langcities_common::merge::Merge;
use langcities_config::error::{LcConfigError, LcConfigErrorTrait};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialServerConfig {
    pub bind_address: Option<String>,
    pub bind_port: Option<u16>,
}

impl PartialServerConfig {
    pub fn modify_env_provider(base: Env) -> Env {
        base.map(|k| {
            if k.starts_with("SERVER_") {
                Uncased::new(format!("server.{}", &k[7..]))
            } else {
                k.into()
            }
        })
    }

    pub fn augment_args_with_prefix(cmd: clap::Command, prefix: impl AsRef<str>) -> clap::Command {
        let prefix = prefix.as_ref();
        let p = |f: &str| format!("{}{}", prefix, f);

        cmd.arg(Arg::new("bind_address").long(p("bind-address")))
            .arg(
                Arg::new("bind_port")
                    .long(p("bind-port"))
                    .value_parser(value_parser!(u16)),
            )
    }
}

impl Args for PartialServerConfig {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        Self::augment_args_with_prefix(cmd, "server-")
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        Self::augment_args(cmd)
    }
}

impl FromArgMatches for PartialServerConfig {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        let mut me = Self::default();
        me.update_from_arg_matches(matches)?;
        Ok(me)
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        matches
            .get_one::<String>("bind_address")
            .map(|a| self.bind_address.replace(a.clone()));
        matches
            .get_one::<u16>("bind_port")
            .map(|p| self.bind_port.replace(*p));
        Ok(())
    }
}

impl Merge<PartialServerConfig> for PartialServerConfig {
    fn merge_with(&mut self, rhs: PartialServerConfig) {
        self.bind_address.merge_with(rhs.bind_address);
        self.bind_port.merge_with(rhs.bind_port);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind_address: IpAddr,
    pub bind_port: u16,
}

impl ServerConfig {
    pub fn from_partial(
        partial: impl Into<PartialServerConfig>,
        default_port: u16,
    ) -> Result<Self, LcConfigError> {
        let partial = partial.into();
        let bind_address = partial
            .bind_address
            .map(|a| IpAddr::from_str(&a))
            .transpose()
            .map_err(|e| LcConfigError::bad_parse(Some(e.into())))?
            .unwrap_or_else(|| Ipv4Addr::new(0, 0, 0, 0).into());
        let bind_port = partial.bind_port.unwrap_or(default_port);
        Ok(Self {
            bind_address,
            bind_port,
        })
    }

    pub fn bind_host(&self) -> String {
        format!("{}:{}", self.bind_address, self.bind_port)
    }
}
