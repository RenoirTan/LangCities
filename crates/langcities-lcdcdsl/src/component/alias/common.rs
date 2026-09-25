use std::{error::Error as StdError, fmt::Display, ops::AddAssign, str::FromStr};

use crate::{
    component::{Id, Slug, UserAlias},
    error::{DslError, DslErrorTrait},
    impl_deser_fromfullid, impl_ser_tofullid,
};

pub const IDENTIFIER_PREFIX: &'static str = "$";

pub trait BadParseError: StdError {
    fn bad_prefix(s: &str) -> Self;
}

impl BadParseError for DslError {
    fn bad_prefix(s: &str) -> Self {
        Self::bad_value_of(s)
    }
}

pub trait FromIdentifier: FromStr
where
    <Self as FromStr>::Err: BadParseError,
{
    fn from_partial(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        s.parse::<Self>()
    }

    // TODO: replace &str with Pattern when stabilised (11 years and counting)
    fn from_prefixed(s: &str, prefix: &str) -> Result<Self, <Self as FromStr>::Err> {
        let Some(stripped) = s.strip_prefix(prefix) else {
            return Err(<Self as FromStr>::Err::bad_prefix(s));
        };
        Self::from_partial(stripped)
    }
}

impl<T, E> FromIdentifier for T
where
    T: FromStr<Err = E>,
    E: BadParseError,
{
}

pub trait FromFullIdentifier: FromIdentifier
where
    <Self as FromStr>::Err: BadParseError,
{
    fn from_full(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        Self::from_prefixed(s, IDENTIFIER_PREFIX)
    }
}

impl<T> FromFullIdentifier for T
where
    T: FromIdentifier,
    <Self as FromStr>::Err: BadParseError,
{
}

pub trait ToIdentifier: ToString {
    fn to_partial(&self) -> String {
        self.to_string()
    }

    fn to_prefixed(&self, prefix: &str) -> String {
        let mut s = prefix.to_string();
        s.add_assign(&self.to_partial());
        s
    }
}

impl<T> ToIdentifier for T where T: ToString {}

pub trait ToFullIdentifier: ToIdentifier {
    fn to_full(&self) -> String {
        self.to_prefixed(IDENTIFIER_PREFIX)
    }
}

impl<T> ToFullIdentifier for T where T: ToIdentifier {}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SimpleAlias {
    Id(Id),
    Slug(Slug),
}

impl Display for SimpleAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(id) => id.fmt(f),
            Self::Slug(slug) => slug.fmt(f),
        }
    }
}

impl FromStr for SimpleAlias {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Id>()
            .map(|id| Self::Id(id))
            .or_else(|_| s.parse::<Slug>().map(|slug| Self::Slug(slug)))
    }
}

impl_deser_fromfullid!(SimpleAlias);
impl_ser_tofullid!(SimpleAlias);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SimpleAliasedResource {
    pub slug: Slug,
    pub user_alias: UserAlias,
}

impl Display for SimpleAliasedResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.slug, self.user_alias)
    }
}

impl FromStr for SimpleAliasedResource {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split("@");
        let slug = parts
            .next()
            .ok_or_else(|| DslError::bad_value_of(s))
            .map(|first| first.parse::<Slug>())
            .flatten()?;
        let user_alias = parts
            .next()
            .ok_or_else(|| DslError::bad_value_of(s))
            .map(|second| second.parse::<SimpleAlias>())
            .flatten()?;
        Ok(Self { slug, user_alias })
    }
}

impl_ser_tofullid!(SimpleAliasedResource);
impl_deser_fromfullid!(SimpleAliasedResource);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SimpleResourceAlias {
    Id(Id),
    Alias(SimpleAliasedResource),
    Slug(Slug),
}

impl Display for SimpleResourceAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(a) => a.fmt(f),
            Self::Alias(a) => a.fmt(f),
            Self::Slug(a) => a.fmt(f),
        }
    }
}

impl FromStr for SimpleResourceAlias {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Id>()
            .map(Self::Id)
            .or_else(|_| s.parse::<Slug>().map(Self::Slug))
            .or_else(|_| s.parse::<SimpleAliasedResource>().map(Self::Alias))
    }
}

impl_ser_tofullid!(SimpleResourceAlias);
impl_deser_fromfullid!(SimpleResourceAlias);
