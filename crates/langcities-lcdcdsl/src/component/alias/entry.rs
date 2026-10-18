use std::{fmt::Display, str::FromStr};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use crate::{
    component::{
        AliasParts, FromFullIdentifier, Id, Index, QualifiedAlias, ToFullIdentifier, VernacularRef,
    },
    error::{DslError, DslErrorTrait},
    impl_deser_fromfullid, impl_ser_tofullid,
};

const ENTRY_TYPE_TAG: &str = "e";

/// Qualified entry identity: `<vernacular>.<index>`.
///
/// `index` is optional because the same payload is used inside an entry field alias,
/// where `_` means the entry that owns the current field.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct QualifiedEntry {
    pub vernacular: VernacularRef,
    pub index: Option<Index>,
}

impl Display for QualifiedEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.vernacular.fmt(f)?;
        if let Some(index) = &self.index {
            write!(f, ".{}", index)
        } else {
            write!(f, "._")
        }
    }
}

/// Untagged reference to an entry. Used inside entry field aliases.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub enum EntryRef {
    Id(Id),
    Qualified(QualifiedEntry),
}

impl Display for EntryRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(a) => a.fmt(f),
            Self::Qualified(a) => a.fmt(f),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub enum EntryAlias {
    Id(Id),
    Qualified(QualifiedAlias<QualifiedEntry>),
}

impl Display for EntryAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(a) => write!(f, "{ENTRY_TYPE_TAG}:{}", a),
            Self::Qualified(a) => {
                if a.is_explicit() {
                    write!(f, "{ENTRY_TYPE_TAG}:")?;
                }
                a.value.fmt(f)
            }
        }
    }
}

impl FromStr for EntryAlias {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = AliasParts::parse(s)?;
        let explicit_type = parts.explicit_type(ENTRY_TYPE_TAG, s)?;
        match parts.tail.len() {
            0 => {
                if !explicit_type {
                    return Err(DslError::bad_value_of(s));
                }
                parts.require_no_owner(s)?;
                Ok(Self::Id(parts.first.parse::<Id>()?))
            }
            1 => {
                let index = parts.tail[0].parse::<Index>()?;
                let vernacular = VernacularRef::from_parts(parts.first, parts.owner)?;
                Ok(Self::Qualified(QualifiedAlias::new(
                    QualifiedEntry {
                        vernacular,
                        index: Some(index),
                    },
                    explicit_type,
                )))
            }
            _ => Err(DslError::bad_value_of(s)),
        }
    }
}

impl FromFullIdentifier for EntryAlias {}
impl ToFullIdentifier for EntryAlias {}

impl_ser_tofullid!(EntryAlias);
impl_deser_fromfullid!(EntryAlias);

#[cfg(test)]
mod test {
    use crate::component::{EntryAlias, FromFullIdentifier, ToFullIdentifier};

    #[test]
    fn test_valid_entry_aliases() {
        let cases = [
            "$e:456",
            "$e:123.100",
            "$e:mt.100",
            "$e:mt@derperor.100",
            "$e:mt@123.100",
            "$123.100",
            "$mt.100",
            "$mt@derperor.100",
            "$mt@123.100",
        ];

        for case in cases {
            let alias = EntryAlias::from_full(case).unwrap();
            assert_eq!(alias.to_full(), case);
        }
    }

    #[test]
    fn test_old_tag_shaped_slug_is_an_entry() {
        for case in ["$ef.123", "$sc.123"] {
            let alias = EntryAlias::from_full(case).unwrap();
            assert_eq!(alias.to_full(), case);
        }
    }

    #[test]
    fn test_invalid_entry_aliases() {
        let cases = [
            "$456",
            "$e:",
            "$e:mt",
            "$e:mt@derperor",
            "$e:123@derperor.100",
            "$e:mt.100.ipa",
            "$e:mt._.100",
            "$mt.sc.ot_mt",
        ];
        for case in cases {
            assert!(EntryAlias::from_full(case).is_err(), "{case}");
        }
    }
}
