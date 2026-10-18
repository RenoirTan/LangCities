use std::{
    error::Error as StdError,
    fmt::Display,
    hash::{Hash, Hasher},
    ops::{AddAssign, Deref},
    str::FromStr,
};

use smallvec::SmallVec;
#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use crate::{
    component::{Slug, UserAlias},
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

/// Top-level qualified alias. The flag is deliberately private:
/// it only exists so that an implicit alias can be printed back with the same surface syntax.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct QualifiedAlias<T> {
    pub value: T,
    explicit_type: bool,
}

impl<T> QualifiedAlias<T> {
    pub(crate) fn new(value: T, explicit_type: bool) -> Self {
        Self {
            value,
            explicit_type,
        }
    }

    pub fn is_explicit(&self) -> bool {
        self.explicit_type
    }
}

impl<T: PartialEq> PartialEq for QualifiedAlias<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq> Eq for QualifiedAlias<T> {}

impl<T: Hash> Hash for QualifiedAlias<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T> Deref for QualifiedAlias<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Parsed form of a partial alias, i.e. after the leading `$` was removed.
///
/// `$v:mt@user.123` becomes
/// `tag=Some("v"), first="mt", owner=Some("user"), tail=["123"]`.
#[derive(Clone, Debug)]
pub(crate) struct AliasParts<'a> {
    pub tag: Option<&'a str>,
    pub first: &'a str,
    pub owner: Option<&'a str>,
    pub tail: SmallVec<[&'a str; 3]>,
}

impl<'a> AliasParts<'a> {
    pub fn parse(s: &'a str) -> Result<Self, DslError> {
        let err = || DslError::bad_value_of(s);

        let (tag, rest) = match s.split_once(':') {
            Some((tag, rest)) => {
                tag.parse::<Slug>().map_err(|_| err())?;
                (Some(tag), rest)
            }
            None => (None, s),
        };
        if rest.is_empty() {
            return Err(err());
        }

        let mut segments = rest.split('.');
        let first_segment = segments.next().ok_or_else(err)?;
        if first_segment.is_empty() {
            return Err(err());
        }
        let (first, owner) = match first_segment.split_once('@') {
            Some((first, owner)) => {
                if first.is_empty() || owner.is_empty() || owner.contains('@') {
                    return Err(err());
                }
                (first, Some(owner))
            }
            None => (first_segment, None),
        };

        let mut tail = SmallVec::new();
        for segment in segments {
            if segment.is_empty() {
                return Err(err());
            }
            tail.push(segment);
        }

        Ok(Self {
            tag,
            first,
            owner,
            tail,
        })
    }

    pub fn explicit_type(&self, expected: &str, s: &str) -> Result<bool, DslError> {
        match self.tag {
            Some(tag) if tag == expected => Ok(true),
            None => Ok(false),
            _ => Err(DslError::bad_value_of(s)),
        }
    }

    pub fn require_no_owner(&self, s: &str) -> Result<(), DslError> {
        if self.owner.is_some() {
            Err(DslError::bad_value_of(s))
        } else {
            Ok(())
        }
    }
}

/// `<slug>[@<user>]`, where an absent user means the current user.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct SimpleQualifiedResource {
    pub slug: Slug,
    pub user_alias: Option<UserAlias>,
}

impl Display for SimpleQualifiedResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(user_alias) = &self.user_alias {
            write!(f, "{}@{}", self.slug, user_alias)
        } else {
            self.slug.fmt(f)
        }
    }
}

impl FromStr for SimpleQualifiedResource {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = AliasParts::parse(s)?;
        if parts.tag.is_some() || !parts.tail.is_empty() {
            return Err(DslError::bad_value_of(s));
        }
        let slug = parts.first.parse::<Slug>()?;
        let user_alias = parts.owner.map(UserAlias::from_bare).transpose()?;
        Ok(Self { slug, user_alias })
    }
}

impl FromFullIdentifier for SimpleQualifiedResource {}
impl ToFullIdentifier for SimpleQualifiedResource {}

impl_ser_tofullid!(SimpleQualifiedResource);
impl_deser_fromfullid!(SimpleQualifiedResource);
