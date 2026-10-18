use std::{fmt::Display, str::FromStr};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use crate::{
    component::{
        AliasParts, EntryFieldAlias, FromFullIdentifier, SoundChangeAlias, ToFullIdentifier,
    },
    error::{DslError, DslErrorTrait},
    impl_deser_fromfullid, impl_ser_tofullid,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub enum DepAlias {
    EntryField(EntryFieldAlias),
    SoundChange(SoundChangeAlias),
}

impl DepAlias {
    pub fn partition<E, S>(self, ef: &mut E, sc: &mut S)
    where
        E: Extend<EntryFieldAlias>,
        S: Extend<SoundChangeAlias>,
    {
        match self {
            Self::EntryField(a) => ef.extend(Some(a)),
            Self::SoundChange(a) => sc.extend(Some(a)),
        }
    }

    pub fn partition_all<I, E, S>(iter: I, ef: &mut E, sc: &mut S)
    where
        I: IntoIterator<Item = DepAlias>,
        E: Extend<EntryFieldAlias>,
        S: Extend<SoundChangeAlias>,
    {
        for alias in iter {
            alias.partition(ef, sc);
        }
    }
}

impl Display for DepAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EntryField(a) => a.fmt(f),
            Self::SoundChange(a) => a.fmt(f),
        }
    }
}

impl FromStr for DepAlias {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = AliasParts::parse(s)?;
        match parts.tag {
            Some("sc") => s.parse::<SoundChangeAlias>().map(Self::SoundChange),
            Some("ef") | None => s.parse::<EntryFieldAlias>().map(Self::EntryField),
            Some(_) => Err(DslError::bad_value_of(s)),
        }
    }
}

impl FromFullIdentifier for DepAlias {}
impl ToFullIdentifier for DepAlias {}

impl_ser_tofullid!(DepAlias);
impl_deser_fromfullid!(DepAlias);

#[cfg(test)]
mod test {
    use crate::component::{DepAlias, FromFullIdentifier, ToFullIdentifier};

    #[test]
    fn test_valid_dep_aliases() {
        let cases = [
            "$ef:789",
            "$456.ipa",
            "$123.100.ipa",
            "$mt.100.ipa",
            "$mt@derperor.100.ipa",
            "$ef:_.ipa",
            "$ef:mt._.ipa",
            "$sc:321",
            "$sc:123.ot_mt",
            "$sc:mt@derperor.ot_mt",
        ];
        for case in cases {
            let alias = DepAlias::from_full(case).unwrap();
            assert_eq!(alias.to_full(), case);
        }
    }

    #[test]
    fn test_invalid_dep_aliases() {
        let cases = [
            "$123",
            "$ef.123",
            "$sc.123",
            "$mt.sc.ot_mt",
            "$mt@derperor.sc.ot_mt",
            "$sc:mt.ot_mt.extra",
        ];
        for case in cases {
            assert!(DepAlias::from_full(case).is_err(), "{case}");
        }
    }
}
