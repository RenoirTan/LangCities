use std::{
    fmt::Display,
    hash::{Hash, Hasher},
    str::FromStr,
};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use crate::{
    component::{AliasParts, FromFullIdentifier, Id, Slug, ToFullIdentifier},
    error::{DslError, DslErrorTrait},
    impl_deser_fromfullid, impl_ser_tofullid,
};

const USER_TYPE_TAG: &str = "u";
const USER_SUGAR_PREFIX: &str = "@";
const USER_TYPE_PREFIX: &str = "$u:";

#[derive(Clone, Debug)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct UserId {
    id: Id,
    explicit_type: bool,
}

impl UserId {
    pub(crate) fn new(id: Id, explicit_type: bool) -> Self {
        Self { id, explicit_type }
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    fn is_explicit(&self) -> bool {
        self.explicit_type
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

impl PartialEq for UserId {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for UserId {}

impl Hash for UserId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct Username {
    username: Slug,
    explicit_type: bool,
}

impl Username {
    pub(crate) fn new(username: Slug, explicit_type: bool) -> Self {
        Self {
            username,
            explicit_type,
        }
    }

    pub fn username(&self) -> &Slug {
        &self.username
    }

    fn is_explicit(&self) -> bool {
        self.explicit_type
    }
}

impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.username.fmt(f)
    }
}

impl PartialEq for Username {
    fn eq(&self, other: &Self) -> bool {
        self.username == other.username
    }
}

impl Eq for Username {}

impl Hash for Username {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.username.hash(state);
    }
}

/// A user is the one resource that is not split into ID and Qualified;
/// `Username` is its non-ID form.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub enum UserAlias {
    Id(UserId),
    Username(Username),
}

impl UserAlias {
    /// Bare form used for the owner part of `slug@owner`.
    pub(crate) fn from_bare(s: &str) -> Result<Self, DslError> {
        Self::from_bare_with(s, false)
    }

    fn from_bare_with(s: &str, explicit_type: bool) -> Result<Self, DslError> {
        if let Ok(id) = s.parse::<Id>() {
            return Ok(Self::Id(UserId::new(id, explicit_type)));
        }
        let username = s.parse::<Slug>()?;
        Ok(Self::Username(Username::new(username, explicit_type)))
    }
}

impl Display for UserAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(a) => a.fmt(f),
            Self::Username(a) => a.fmt(f),
        }
    }
}

impl FromStr for UserAlias {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = AliasParts::parse(s)?;
        if parts.owner.is_some() || !parts.tail.is_empty() {
            return Err(DslError::bad_value_of(s));
        }
        let explicit_type = match parts.tag {
            Some(USER_TYPE_TAG) => true,
            None => false,
            _ => return Err(DslError::bad_value_of(s)),
        };
        Self::from_bare_with(parts.first, explicit_type)
    }
}

impl FromFullIdentifier for UserAlias {
    fn from_full(s: &str) -> Result<Self, Self::Err> {
        if let Some(rest) = s.strip_prefix(USER_SUGAR_PREFIX) {
            Self::from_bare_with(rest, false)
        } else if let Some(rest) = s.strip_prefix(USER_TYPE_PREFIX) {
            Self::from_bare_with(rest, true)
        } else {
            Err(DslError::bad_value_of(s))
        }
    }
}

impl ToFullIdentifier for UserAlias {
    fn to_full(&self) -> String {
        match self {
            Self::Id(a) if a.is_explicit() => format!("{USER_TYPE_PREFIX}{}", a.id),
            Self::Id(a) => format!("{USER_SUGAR_PREFIX}{}", a.id),
            Self::Username(a) if a.is_explicit() => {
                format!("{USER_TYPE_PREFIX}{}", a.username)
            }
            Self::Username(a) => format!("{USER_SUGAR_PREFIX}{}", a.username),
        }
    }
}

impl_ser_tofullid!(UserAlias);
impl_deser_fromfullid!(UserAlias);

#[cfg(test)]
mod test {
    use crate::component::{FromFullIdentifier, ToFullIdentifier, UserAlias};

    #[test]
    fn test_valid_user_aliases() {
        let cases = ["@123", "@derperor", "$u:123", "$u:derperor"];

        for case in cases {
            let alias = UserAlias::from_full(case).unwrap();
            assert_eq!(alias.to_full(), case);
        }
    }

    #[test]
    fn test_invalid_user_aliases() {
        let cases = [
            "$123",
            "$derperor",
            "123",
            "derperor",
            "@",
            "$u:",
            "$u:123@x",
            "$135",
        ];
        for case in cases {
            assert!(UserAlias::from_full(case).is_err(), "{case}");
        }
    }

    #[test]
    fn test_user_alias_equality_ignores_sugar() {
        let sugar = UserAlias::from_full("@123").unwrap();
        let explicit = UserAlias::from_full("$u:123").unwrap();
        assert_eq!(sugar, explicit);
    }
}
