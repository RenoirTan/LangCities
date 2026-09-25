use std::{fmt::Display, str::FromStr};

use smallvec::SmallVec;

use crate::{
    component::{Id, Index, VernacularAlias},
    error::{DslError, DslErrorTrait},
    impl_deser_fromfullid, impl_ser_tofullid,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AliasedEntry {
    pub vernacular_alias: VernacularAlias,
    pub index: Index,
}

impl Display for AliasedEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.vernacular_alias, self.index)
    }
}

impl FromStr for AliasedEntry {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: SmallVec<[&str; 2]> = s.rsplitn(2, '.').collect();
        if parts.len() <= 1 {
            return Err(DslError::bad_value_of(s));
        }
        let index = parts[0].parse::<Index>()?;
        let vernacular_alias = parts[1].parse::<VernacularAlias>()?;
        Ok(AliasedEntry {
            vernacular_alias,
            index,
        })
    }
}

impl_deser_fromfullid!(AliasedEntry);
impl_ser_tofullid!(AliasedEntry);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EntryAlias {
    Id(Id),
    Alias(AliasedEntry),
}

impl Display for EntryAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(a) => a.fmt(f),
            Self::Alias(a) => a.fmt(f),
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

impl_deser_fromfullid!(EntryAlias);
impl_ser_tofullid!(EntryAlias);

#[cfg(test)]
mod test {
    use crate::component::{EntryAlias, FromFullIdentifier, ToFullIdentifier};

    #[test]
    fn test_valid_entry_alias() {
        let cases = ["$lang@me.123", "$lang.456", "$789", "$123.456"];

        for case in cases {
            let result = EntryAlias::from_full(case).unwrap();
            assert_eq!(result.to_full(), case);
        }
    }

    #[test]
    fn test_invalid_entry_alias() {
        let cases = [
            "lang@me.123",
            "lang.456",
            "789",
            "123.456",
            "$lang@.234",
            "$@missing.345",
            "$ouch.",
            "$.567",
            "$.",
        ];
        for case in cases {
            assert!(EntryAlias::from_full(case).is_err());
        }
    }
}
