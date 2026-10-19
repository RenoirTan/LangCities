use std::{fmt::Display, str::FromStr};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use crate::{
    component::{
        AliasParts, FromFullIdentifier, Id, QualifiedAlias, SimpleQualifiedResource, Slug,
        ToFullIdentifier, UserAlias,
    },
    error::{DslError, DslErrorTrait},
    impl_deser_fromfullid, impl_ser_tofullid,
};

const VERNACULAR_TYPE_TAG: &str = "v";

pub type QualifiedVernacular = SimpleQualifiedResource;

/// Untagged reference to a vernacular.
/// Used when a vernacular appears inside an entry,
/// entry field or sound change alias.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub enum VernacularRef {
    Id(Id),
    Qualified(QualifiedVernacular),
}

impl VernacularRef {
    pub(crate) fn from_parts(first: &str, owner: Option<&str>) -> Result<Self, DslError> {
        if owner.is_none() {
            if let Ok(id) = first.parse::<Id>() {
                return Ok(Self::Id(id));
            }
        }
        let slug = first.parse::<Slug>()?;
        let user_alias = owner.map(UserAlias::from_bare).transpose()?;
        Ok(Self::Qualified(SimpleQualifiedResource {
            slug,
            user_alias,
        }))
    }
}

impl Display for VernacularRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(a) => a.fmt(f),
            Self::Qualified(a) => a.fmt(f),
        }
    }
}

impl FromStr for VernacularRef {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = AliasParts::parse(s)?;
        if parts.tag.is_some() || !parts.tail.is_empty() {
            return Err(DslError::bad_value_of(s));
        }
        Self::from_parts(parts.first, parts.owner)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub enum VernacularAlias {
    Id(Id),
    Qualified(QualifiedAlias<QualifiedVernacular>),
}

impl Display for VernacularAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(a) => write!(f, "{VERNACULAR_TYPE_TAG}:{}", a),
            Self::Qualified(a) => {
                if a.is_explicit() {
                    write!(f, "{VERNACULAR_TYPE_TAG}:")?;
                }
                a.value.fmt(f)
            }
        }
    }
}

impl FromStr for VernacularAlias {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = AliasParts::parse(s)?;
        let explicit_type = parts.explicit_type(VERNACULAR_TYPE_TAG, s)?;
        if !parts.tail.is_empty() {
            return Err(DslError::bad_value_of(s));
        }
        match VernacularRef::from_parts(parts.first, parts.owner)? {
            VernacularRef::Id(id) if explicit_type => Ok(Self::Id(id)),
            VernacularRef::Id(_) => Err(DslError::bad_value_of(s)),
            VernacularRef::Qualified(value) => {
                Ok(Self::Qualified(QualifiedAlias::new(value, explicit_type)))
            }
        }
    }
}

impl From<VernacularRef> for VernacularAlias {
    fn from(value: VernacularRef) -> Self {
        match value {
            VernacularRef::Id(id) => Self::Id(id),
            VernacularRef::Qualified(alias) => Self::Qualified(QualifiedAlias::from_value(alias)),
        }
    }
}

impl Into<VernacularRef> for VernacularAlias {
    fn into(self) -> VernacularRef {
        match self {
            Self::Id(id) => VernacularRef::Id(id),
            Self::Qualified(alias) => VernacularRef::Qualified(alias.value),
        }
    }
}

impl FromFullIdentifier for VernacularAlias {}
impl ToFullIdentifier for VernacularAlias {}

impl_ser_tofullid!(VernacularAlias);
impl_deser_fromfullid!(VernacularAlias);

#[cfg(test)]
mod test {
    use crate::component::{FromFullIdentifier, ToFullIdentifier, VernacularAlias, VernacularRef};

    #[test]
    fn test_valid_vernacular_aliases() {
        let cases = [
            "$v:123",
            "$v:mt",
            "$v:mt@derperor",
            "$v:mt@123",
            "$mt",
            "$mt@derperor",
            "$mt@123",
        ];

        for case in cases {
            let alias = VernacularAlias::from_full(case).unwrap();
            assert_eq!(alias.to_full(), case);
        }
    }

    #[test]
    fn test_invalid_vernacular_aliases() {
        let cases = [
            "$123",
            "$v:",
            "$v:123@derperor",
            "$v:mt.123",
            "$mt.123",
            "$u:123",
            "$v:123.456",
        ];
        for case in cases {
            assert!(VernacularAlias::from_full(case).is_err(), "{case}");
        }
    }

    #[test]
    fn test_vernacular_ref_is_untagged() {
        let cases = ["123", "mt", "mt@derperor", "mt@123"];
        for case in cases {
            let alias = case.parse::<VernacularRef>().unwrap();
            assert_eq!(alias.to_string(), case);
        }
    }
}
