use std::{error::Error as StdError, fmt::Display, ops::AddAssign, str::FromStr};

use smallvec::SmallVec;

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

fn join_path(path: &[Slug]) -> String {
    path.iter()
        .map(|s| (**s).clone())
        .reduce(|acc, s| acc + "." + &s)
        .unwrap_or_default()
}

/// Represents <slug>[@<user>]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ComplexHead {
    Alias(SimpleAliasedResource),
    Slug(Slug),
}

impl Display for ComplexHead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Alias(a) => a.fmt(f),
            Self::Slug(a) => a.fmt(f),
        }
    }
}

/// Represents <id>[.<slug>]*
///
/// `N+1` is the maximum length of the complex path.
/// `N` is the maximum number of dots '.'
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComplexIdAliasedResource<const N: usize> {
    pub id: Id,
    pub path: SmallVec<[Slug; N]>,
}

impl<const N: usize> Display for ComplexIdAliasedResource<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.id, join_path(&self.path))
    }
}

/// Represents <slug>[@<user>][.<slug>]*
///
/// `N+1` is the length of the complex path.
/// `N` is the number of dots '.'
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComplexHeadAliasedResource<const N: usize> {
    pub head: ComplexHead,
    pub path: [Slug; N],
}

impl<const N: usize> Display for ComplexHeadAliasedResource<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.head, join_path(&self.path))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ComplexAliasedResource<const N: usize> {
    IdFormat(ComplexIdAliasedResource<N>),
    HeadFormat(ComplexHeadAliasedResource<N>),
}

impl<const N: usize> Display for ComplexAliasedResource<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IdFormat(a) => a.fmt(f),
            Self::HeadFormat(a) => a.fmt(f),
        }
    }
}

impl<const N: usize> FromStr for ComplexAliasedResource<N> {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts: SmallVec<[&str; N]> = s.rsplitn(N, ".").collect();
        let len = parts.len();
        // extract head from parts
        // leave path inside parts
        // rsplit, so first component is the last element
        let raw_head = if let Some(first) = parts.pop() {
            if len == N {
                let mut splat = first.rsplitn(2, ".");
                let maybe_head = splat.next().unwrap(); // at least one string per slice
                let actual_head = splat.next();
                if let Some(actual_head) = actual_head {
                    parts.push(maybe_head);
                    actual_head
                } else {
                    maybe_head
                }
            } else {
                first
            }
        } else {
            return Err(DslError::bad_value_of(s));
        };
        let sra = SimpleResourceAlias::from_partial(raw_head)?;
        // Complex head must have full length path
        match sra {
            SimpleResourceAlias::Alias(_) | SimpleResourceAlias::Slug(_) if parts.len() < N => {
                return Err(DslError::bad_value_of(s));
            }
            _ => {}
        }
        let path = parts
            .into_iter()
            .try_fold(SmallVec::<[Slug; N]>::new(), |mut acc, s| {
                let slug = Slug::from_partial(s)?;
                acc.push(slug);
                Ok(acc)
            })?;
        Ok(match sra {
            SimpleResourceAlias::Id(id) => {
                ComplexAliasedResource::IdFormat(ComplexIdAliasedResource { id, path })
            }
            SimpleResourceAlias::Alias(alias) => {
                ComplexAliasedResource::HeadFormat(ComplexHeadAliasedResource {
                    head: ComplexHead::Alias(alias),
                    path: path.into_inner().unwrap(),
                })
            }
            SimpleResourceAlias::Slug(slug) => {
                ComplexAliasedResource::HeadFormat(ComplexHeadAliasedResource {
                    head: ComplexHead::Slug(slug),
                    path: path.into_inner().unwrap(),
                })
            }
        })
    }
}

impl<const N: usize> serde::Serialize for ComplexAliasedResource<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_full())
    }
}

impl<'de, const N: usize> serde::Deserialize<'de> for ComplexAliasedResource<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)
            .map(|s| {
                ComplexAliasedResource::from_full(&s)
                    .map_err(|e| <D::Error as serde::de::Error>::custom(e))
            })
            .flatten()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ComplexResourceAlias<const N: usize> {
    Id(Id),
    Alias(ComplexAliasedResource<N>),
}

impl<const N: usize> Display for ComplexResourceAlias<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(a) => a.fmt(f),
            Self::Alias(a) => a.fmt(f),
        }
    }
}

impl<const N: usize> FromStr for ComplexResourceAlias<N> {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Id>().map(ComplexResourceAlias::Id).or_else(|_| {
            s.parse::<ComplexAliasedResource<N>>()
                .map(ComplexResourceAlias::Alias)
        })
    }
}

impl<const N: usize> serde::Serialize for ComplexResourceAlias<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_full())
    }
}

impl<'de, const N: usize> serde::Deserialize<'de> for ComplexResourceAlias<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)
            .map(|s| {
                ComplexResourceAlias::from_full(&s)
                    .map_err(|e| <D::Error as serde::de::Error>::custom(e))
            })
            .flatten()
    }
}

#[cfg(test)]
mod test {
    use crate::component::{
        ComplexResourceAlias, FromFullIdentifier, SimpleAlias, SimpleResourceAlias,
        ToFullIdentifier, ToIdentifier,
    };

    #[test]
    fn test_valid_simple_aliases() {
        let cases = ["$me", "$1", "$2489", "$user"];
        for case in cases {
            let alias = SimpleAlias::from_full(case).unwrap();
            assert_eq!(alias.to_full(), case);
            assert_eq!(alias.to_partial(), &case[1..])
        }
    }

    #[test]
    fn test_invalid_simple_aliases() {
        let cases = ["oof", "123", "$$hi", "$0emfe", "$hello "];
        for case in cases {
            assert!(SimpleAlias::from_full(case).is_err());
        }
    }

    #[test]
    fn test_valid_simple_resource_aliases() {
        let cases = ["$lang@me", "$rust@123", "$slug", "$123"];
        for case in cases {
            let alias = SimpleResourceAlias::from_full(case).unwrap();
            assert_eq!(alias.to_full(), case);
            assert_eq!(alias.to_partial(), &case[1..])
        }
    }

    #[test]
    fn test_invalid_simple_resource_aliases() {
        let cases = [
            "lang@me", "rust@123", "slug", "123", "$$hi", "$0emfe", "$hello ", "$1@23",
        ];
        for case in cases {
            assert!(SimpleResourceAlias::from_full(case).is_err());
        }
    }

    #[test]
    fn test_valid_complex_resource_aliases() {
        fn test<const N: usize>(case: &str) {
            let alias = ComplexResourceAlias::<N>::from_full(case).unwrap();
            assert_eq!(alias.to_full(), case);
            assert_eq!(alias.to_partial(), &case[1..])
        }
        test::<1>("$lang@me.hi");
    }
}
