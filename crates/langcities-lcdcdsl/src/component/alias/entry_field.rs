use std::{fmt::Display, str::FromStr};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use crate::{
    component::{
        AliasParts, EntryRef, FromFullIdentifier, Id, Index, QualifiedAlias, QualifiedEntry, Slug,
        ToFullIdentifier, VernacularRef,
    },
    error::{DslError, DslErrorTrait},
    impl_deser_fromfullid, impl_ser_tofullid,
};

const ENTRY_FIELD_TYPE_TAG: &str = "ef";

/// Qualified entry field identity: `<entry>.<field_slug>`.
///
/// `entry == None` is the fully relative form `_.ipa`; a qualified entry with
/// `index == None` is the wildcard form `mt._.ipa`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct QualifiedEntryField {
    pub entry: Option<EntryRef>,
    pub slug: Slug,
}

impl Display for QualifiedEntryField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(entry) = &self.entry {
            write!(f, "{}.{}", entry, self.slug)
        } else {
            write!(f, "_.{}", self.slug)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub enum EntryFieldAlias {
    Id(Id),
    Qualified(QualifiedAlias<QualifiedEntryField>),
}

impl EntryFieldAlias {
    /// Reject wildcard and fully relative forms.
    /// Used by API endpoints that have no current entry/field context.
    pub fn require_concrete(self) -> Result<Self, DslError> {
        if let Self::Qualified(a) = &self {
            match &a.value.entry {
                None => return Err(DslError::bad_value_of("relative entry field alias")),
                Some(EntryRef::Qualified(entry)) if entry.index.is_none() => {
                    return Err(DslError::bad_value_of("relative entry field alias"));
                }
                _ => {}
            }
        }
        Ok(self)
    }
}

impl Display for EntryFieldAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(a) => write!(f, "{ENTRY_FIELD_TYPE_TAG}:{}", a),
            Self::Qualified(a) => {
                if a.is_explicit() {
                    write!(f, "{ENTRY_FIELD_TYPE_TAG}:")?;
                }
                a.value.fmt(f)
            }
        }
    }
}

impl FromStr for EntryFieldAlias {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = AliasParts::parse(s)?;
        let explicit_type = parts.explicit_type(ENTRY_FIELD_TYPE_TAG, s)?;
        match parts.tail.len() {
            0 => {
                if !explicit_type {
                    return Err(DslError::bad_value_of(s));
                }
                parts.require_no_owner(s)?;
                Ok(Self::Id(parts.first.parse::<Id>()?))
            }
            1 => {
                parts.require_no_owner(s)?;
                let slug = parts.tail[0].parse::<Slug>()?;
                let entry = if parts.first == "_" {
                    None
                } else {
                    Some(EntryRef::Id(parts.first.parse::<Id>()?))
                };
                Ok(Self::Qualified(QualifiedAlias::new(
                    QualifiedEntryField { entry, slug },
                    explicit_type,
                )))
            }
            2 => {
                let index = if parts.tail[0] == "_" {
                    None
                } else {
                    Some(parts.tail[0].parse::<Index>()?)
                };
                let vernacular = VernacularRef::from_parts(parts.first, parts.owner)?;
                let slug = parts.tail[1].parse::<Slug>()?;
                Ok(Self::Qualified(QualifiedAlias::new(
                    QualifiedEntryField {
                        entry: Some(EntryRef::Qualified(QualifiedEntry { vernacular, index })),
                        slug,
                    },
                    explicit_type,
                )))
            }
            _ => Err(DslError::bad_value_of(s)),
        }
    }
}

impl FromFullIdentifier for EntryFieldAlias {}
impl ToFullIdentifier for EntryFieldAlias {}

impl_ser_tofullid!(EntryFieldAlias);
impl_deser_fromfullid!(EntryFieldAlias);

#[cfg(test)]
mod test {
    use crate::component::{EntryFieldAlias, FromFullIdentifier, ToFullIdentifier};

    #[test]
    fn test_valid_entry_field_aliases() {
        let cases = [
            "$ef:789",
            "$ef:456.ipa",
            "$ef:123.100.ipa",
            "$ef:mt.100.ipa",
            "$ef:mt@derperor.100.ipa",
            "$ef:mt@123.100.ipa",
            "$ef:_.ipa",
            "$ef:mt._.ipa",
            "$ef:123._.ipa",
            "$456.ipa",
            "$123.100.ipa",
            "$mt.100.ipa",
            "$mt@derperor.100.ipa",
            "$mt@123.100.ipa",
            "$_.ipa",
            "$mt._.ipa",
            "$123._.ipa",
        ];

        for case in cases {
            let alias = EntryFieldAlias::from_full(case).unwrap();
            assert_eq!(alias.to_full(), case);
        }
    }

    #[test]
    fn test_invalid_entry_field_aliases() {
        let cases = [
            "$789",
            "$ef:",
            "$ef.789",
            "$ef:mt",
            "$ef:mt.ipa",
            "$ef:123.ipa.ipa",
            "$ef:456@derperor.ipa",
            "$ef:mt.100",
            "$sc.123",
        ];
        for case in cases {
            assert!(EntryFieldAlias::from_full(case).is_err(), "{case}");
        }
    }

    #[test]
    fn test_equality_ignores_explicit_type() {
        let explicit = EntryFieldAlias::from_full("$ef:mt.100.ipa").unwrap();
        let implicit = EntryFieldAlias::from_full("$mt.100.ipa").unwrap();
        assert_eq!(explicit, implicit);
    }

    #[test]
    fn test_require_concrete() {
        for case in [
            "$ef:789",
            "$ef:456.ipa",
            "$ef:123.100.ipa",
            "$ef:mt.100.ipa",
        ] {
            assert!(
                EntryFieldAlias::from_full(case)
                    .unwrap()
                    .require_concrete()
                    .is_ok(),
                "{case}"
            );
        }
        for case in ["$ef:_.ipa", "$ef:mt._.ipa", "$ef:123._.ipa"] {
            assert!(
                EntryFieldAlias::from_full(case)
                    .unwrap()
                    .require_concrete()
                    .is_err(),
                "{case}"
            );
        }
    }
}
