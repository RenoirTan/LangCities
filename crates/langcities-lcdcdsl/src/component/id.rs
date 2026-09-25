use std::{fmt::Display, ops::Deref, str::FromStr};

use serde::{Deserialize, Serialize, de::Visitor};

use crate::error::{DslError, DslErrorTrait};
use crate::impl_deser_fromstr;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Id(i64);
pub type Index = Id;

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
