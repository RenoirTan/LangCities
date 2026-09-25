use std::{fmt::Display, str::FromStr};

use smallvec::SmallVec;

use crate::{
    component::{AliasedEntry, EntryAlias, Id, Index, Slug, VernacularAlias},
    error::{DslError, DslErrorTrait},
    impl_deser_fromfullid, impl_ser_tofullid,
};

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
        let parts: SmallVec<[&str; 2]> = s.rsplitn(2, '.').collect();
        if parts.len() <= 1 {
            return Err(DslError::bad_value_of(s));
        }
        let slug = parts[0].parse::<Slug>()?;
        let entry_alias = parts[1].parse::<EntryAlias>()?;
        Ok(AliasedEntryField { entry_alias, slug })
    }
}

impl_deser_fromfullid!(AliasedEntryField);
impl_ser_tofullid!(AliasedEntryField);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NeedyAliasedEntryField {
    pub vernacular_alias: Option<VernacularAlias>,
    pub entry_index: Option<Index>,
    pub slug: Slug,
}

impl NeedyAliasedEntryField {
    pub fn to_maybe_aliased(self) -> Result<AliasedEntryField, Self> {
        match self {
            Self {
                vernacular_alias: Some(v),
                entry_index: Some(i),
                slug,
            } => Ok(AliasedEntryField {
                entry_alias: EntryAlias::Alias(AliasedEntry {
                    vernacular_alias: v,
                    index: i,
                }),
                slug,
            }),
            s => Err(s),
        }
    }

    pub fn to_aliased(
        self,
        vernacular_alias: VernacularAlias,
        entry_index: Index,
    ) -> AliasedEntryField {
        AliasedEntryField {
            entry_alias: EntryAlias::Alias(AliasedEntry {
                vernacular_alias,
                index: entry_index,
            }),
            slug: self.slug,
        }
    }
}

impl Display for NeedyAliasedEntryField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(a) = &self.vernacular_alias {
            write!(f, "{}.", a)?;
        }
        if let Some(i) = &self.entry_index {
            write!(f, "{}.{}", i, self.slug)
        } else {
            write!(f, "_.{}", self.slug)
        }
    }
}

impl FromStr for NeedyAliasedEntryField {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: SmallVec<[&str; 3]> = s.rsplitn(3, '.').collect();
        if parts.len() <= 1 {
            return Err(DslError::bad_value_of(s));
        }
        let entry_index = if parts[1] == "_" {
            None
        } else {
            Some(parts[1].parse::<Index>()?)
        };
        let slug = parts[0].parse::<Slug>()?;
        let vernacular_alias = parts
            .get(2)
            .map(|v| v.parse::<VernacularAlias>())
            .transpose()?;
        Ok(NeedyAliasedEntryField {
            vernacular_alias,
            entry_index,
            slug,
        })
    }
}

impl_deser_fromfullid!(NeedyAliasedEntryField);
impl_ser_tofullid!(NeedyAliasedEntryField);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NeedyEntryFieldAlias {
    Id(Id),
    Alias(AliasedEntryField),
    Needy(NeedyAliasedEntryField),
}

impl NeedyEntryFieldAlias {
    pub fn to_maybe_alias(self) -> Result<EntryFieldAlias, Self> {
        match self {
            Self::Id(id) => Ok(EntryFieldAlias::Id(id)),
            Self::Alias(alias) => Ok(EntryFieldAlias::Alias(alias)),
            Self::Needy(needy) => match needy.to_maybe_aliased() {
                Ok(alias) => Ok(EntryFieldAlias::Alias(alias)),
                Err(needy) => Err(Self::Needy(needy)),
            },
        }
    }

    pub fn to_alias(
        self,
        vernacular_alias: VernacularAlias,
        entry_index: Index,
    ) -> EntryFieldAlias {
        match self {
            Self::Id(id) => EntryFieldAlias::Id(id),
            Self::Alias(alias) => EntryFieldAlias::Alias(alias),
            Self::Needy(needy) => {
                EntryFieldAlias::Alias(needy.to_aliased(vernacular_alias, entry_index))
            }
        }
    }
}

impl Display for NeedyEntryFieldAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(a) => a.fmt(f),
            Self::Alias(a) => a.fmt(f),
            Self::Needy(a) => a.fmt(f),
        }
    }
}

impl FromStr for NeedyEntryFieldAlias {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Id>().map(Self::Id).or_else(|_| {
            s.parse::<NeedyAliasedEntryField>().map(|n| {
                n.to_maybe_aliased()
                    .map(Self::Alias)
                    .unwrap_or_else(Self::Needy)
            })
        })
    }
}

impl_deser_fromfullid!(NeedyEntryFieldAlias);
impl_ser_tofullid!(NeedyEntryFieldAlias);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EntryFieldAlias {
    Id(Id),
    Alias(AliasedEntryField),
}

impl Display for EntryFieldAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(a) => a.fmt(f),
            Self::Alias(a) => a.fmt(f),
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

impl_deser_fromfullid!(EntryFieldAlias);
impl_ser_tofullid!(EntryFieldAlias);

#[cfg(test)]
mod test {
    use crate::component::{
        EntryFieldAlias, FromFullIdentifier, NeedyEntryFieldAlias, ToFullIdentifier,
    };

    #[test]
    fn test_valid_needy_entry_field_alias() {
        let cases = [
            "$lang@me.123.ipa",
            "$lang.456.transliteration",
            "$789.oof",
            "$123.456.hello",
            "$lang@me._.ipa",
            "$lang._.transliteration",
            "$_.oof",
            "$123._.hello",
        ];

        for case in cases {
            let result = NeedyEntryFieldAlias::from_full(case).unwrap();
            assert_eq!(result.to_full(), case);
        }
    }

    #[test]
    fn test_invalid_needy_entry_field_alias() {
        let cases = [
            "lang@me.123.ipa",
            "lang.456.transliteration",
            "789.oof",
            "123.456.hello",
            "lang@me._.ipa",
            "lang._.transliteration",
            "_.oof",
            "123._.hello",
            "$lang@.123.ipa",
            "$.456.transliteration",
            "$.oof",
            "$3..hello",
            "$@me._.ipa",
            "$lang._.",
            "$_.",
            "$123._.",
        ];

        for case in cases {
            assert!(NeedyEntryFieldAlias::from_full(case).is_err());
        }
    }

    #[test]
    fn test_valid_entry_field_alias() {
        let cases = [
            "$lang@me.123.ipa",
            "$lang.456.transliteration",
            "$789.oof",
            "$123.456.hello",
        ];

        for case in cases {
            let result = EntryFieldAlias::from_full(case).unwrap();
            assert_eq!(result.to_full(), case);
        }
    }

    #[test]
    fn test_invalid_entry_field_alias() {
        let cases = [
            "$lang@me._.ipa",
            "$lang._.transliteration",
            "$_.oof",
            "$123._.hello",
            "lang@me.123.ipa",
            "lang.456.transliteration",
            "789.oof",
            "123.456.hello",
            "lang@me._.ipa",
            "lang._.transliteration",
            "_.oof",
            "123._.hello",
            "$lang@.123.ipa",
            "$.456.transliteration",
            "$.oof",
            "$3..hello",
            "$@me._.ipa",
            "$lang._.",
            "$_.",
            "$123._.",
        ];

        for case in cases {
            assert!(EntryFieldAlias::from_full(case).is_err());
        }
    }
}
