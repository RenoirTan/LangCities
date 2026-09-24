use std::{fmt::Display, ops::Deref, str::FromStr};

use serde::{Deserialize, Serialize, de::Visitor};

use crate::impl_deser_fromstr;
use crate::{
    error::{DslError, DslErrorTrait},
    impl_ser_display,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Id(i64);

impl AsRef<i64> for Id {
    fn as_ref(&self) -> &i64 {
        self
    }
}

impl Deref for Id {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i64> for Id {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl Into<i64> for Id {
    fn into(self) -> i64 {
        self.0
    }
}

impl FromStr for Id {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<i64>()
            .map(|i| Self(i))
            .map_err(DslError::bad_value)
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

struct IdVisitor;

impl<'de> Visitor<'de> for IdVisitor {
    type Value = Id;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("invalid id")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Id(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v <= i64::MAX as u64 {
            self.visit_i64(v as i64)
        } else {
            Err(E::custom("id out of bounds"))
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse::<Id>().map_err(|e| E::custom(e))
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(IdVisitor)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Slug(String);

impl Deref for Slug {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<String> for Slug {
    fn as_ref(&self) -> &String {
        self
    }
}

impl AsRef<str> for Slug {
    fn as_ref(&self) -> &str {
        self
    }
}

impl Display for Slug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Into<String> for Slug {
    fn into(self) -> String {
        self.0
    }
}

impl FromStr for Slug {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let first = s.chars().next().ok_or_else(|| DslError::bad_value_of(s))?;
        if !(first.is_ascii_lowercase() || first == '_') {
            return Err(DslError::bad_value_of(s));
        }
        for c in s.chars().skip(1) {
            if !(c.is_ascii_lowercase() || c.is_digit(10) || c == '_') {
                return Err(DslError::bad_value_of(s));
            }
        }
        Ok(Self(s.to_string()))
    }
}

impl_deser_fromstr!(Slug);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Alias {
    Id(Id),
    Slug(Slug),
}

impl Display for Alias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(id) => id.fmt(f),
            Self::Slug(slug) => slug.fmt(f),
        }
    }
}

impl FromStr for Alias {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Id>()
            .map(|id| Self::Id(id))
            .or_else(|_| s.parse::<Slug>().map(|slug| Self::Slug(slug)))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SlugOwnerId {
    pub slug: Slug,
    pub user_alias: Alias,
}

impl Display for SlugOwnerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.slug, self.user_alias)
    }
}

impl FromStr for SlugOwnerId {
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
            .map(|second| second.parse::<Alias>())
            .flatten()?;
        Ok(Self { slug, user_alias })
    }
}

impl_ser_display!(SlugOwnerId);
impl_deser_fromstr!(SlugOwnerId);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VernacularAlias {
    Id(Id),
    Alias(SlugOwnerId),
    Slug(Slug),
}

impl Display for VernacularAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(id) => id.fmt(f),
            Self::Alias(alias) => alias.fmt(f),
            Self::Slug(slug) => slug.fmt(f),
        }
    }
}

impl FromStr for VernacularAlias {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Id>()
            .map(Self::Id)
            .or_else(|_| s.parse::<Slug>().map(Self::Slug))
            .or_else(|_| s.parse::<SlugOwnerId>().map(Self::Alias))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AliasedEntry {
    pub vernacular_alias: VernacularAlias,
    pub index: Id,
}

impl Display for AliasedEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.vernacular_alias, self.index)
    }
}

impl FromStr for AliasedEntry {
    type Err = DslError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(".");
        let vernacular_alias = parts
            .next()
            .ok_or_else(|| DslError::bad_value_of(s))
            .map(|first| first.parse::<VernacularAlias>())
            .flatten()?;
        let index = parts
            .next()
            .ok_or_else(|| DslError::bad_value_of(s))
            .map(|second| second.parse::<Id>())
            .flatten()?;
        Ok(Self {
            vernacular_alias,
            index,
        })
    }
}

impl_ser_display!(AliasedEntry);
impl_deser_fromstr!(AliasedEntry);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EntryAlias {
    Id(Id),
    Alias(AliasedEntry),
}

impl Display for EntryAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(id) => id.fmt(f),
            Self::Alias(alias) => alias.fmt(f),
        }
    }
}

impl FromStr for EntryAlias {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Id>()
            .map(Self::Id)
            .or_else(|_| s.parse::<AliasedEntry>().map(Self::Alias))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AliasedEntryField {
    pub entry_alias: EntryAlias,
    pub slug: Slug,
}

impl Display for AliasedEntryField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.entry_alias, self.slug)
    }
}

impl FromStr for AliasedEntryField {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.splitn(3, ".").collect();
        let end_index = if parts.len() >= 2 {
            parts.len() - 1
        } else {
            return Err(DslError::bad_value_of(s));
        };
        let entry_alias_part = parts[0..end_index].join(".");
        let entry_alias = entry_alias_part.parse::<EntryAlias>()?;
        let slug = parts[2].parse::<Slug>()?;
        Ok(Self { entry_alias, slug })
    }
}

impl_ser_display!(AliasedEntryField);
impl_deser_fromstr!(AliasedEntryField);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EntryFieldAlias {
    Id(Id),
    Alias(AliasedEntryField),
}

impl Display for EntryFieldAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(id) => id.fmt(f),
            Self::Alias(alias) => alias.fmt(f),
        }
    }
}

impl FromStr for EntryFieldAlias {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Id>()
            .map(Self::Id)
            .or_else(|_| s.parse::<AliasedEntryField>().map(Self::Alias))
    }
}
