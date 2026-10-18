use std::{fmt::Display, str::FromStr};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use crate::{
    component::{AliasParts, FromFullIdentifier, Id, Slug, ToFullIdentifier, VernacularRef},
    error::{DslError, DslErrorTrait},
    impl_deser_fromfullid, impl_ser_tofullid,
};

const SOUND_CHANGE_TYPE_TAG: &str = "sc";

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct QualifiedSoundChange {
    pub vernacular: VernacularRef,
    pub slug: Slug,
}

impl Display for QualifiedSoundChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.vernacular, self.slug)
    }
}

/// Sound changes always require an explicit type tag,
/// so there is no stored `explicit_type` bit here.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub enum SoundChangeAlias {
    Id(Id),
    Qualified(QualifiedSoundChange),
}

impl Display for SoundChangeAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(a) => write!(f, "{SOUND_CHANGE_TYPE_TAG}:{}", a),
            Self::Qualified(a) => write!(f, "{SOUND_CHANGE_TYPE_TAG}:{}", a),
        }
    }
}

impl FromStr for SoundChangeAlias {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = AliasParts::parse(s)?;
        if parts.tag != Some(SOUND_CHANGE_TYPE_TAG) {
            return Err(DslError::bad_value_of(s));
        }
        match parts.tail.len() {
            0 => {
                parts.require_no_owner(s)?;
                Ok(Self::Id(parts.first.parse::<Id>()?))
            }
            1 => {
                let vernacular = VernacularRef::from_parts(parts.first, parts.owner)?;
                let slug = parts.tail[0].parse::<Slug>()?;
                Ok(Self::Qualified(QualifiedSoundChange { vernacular, slug }))
            }
            _ => Err(DslError::bad_value_of(s)),
        }
    }
}

impl FromFullIdentifier for SoundChangeAlias {}
impl ToFullIdentifier for SoundChangeAlias {}

impl_ser_tofullid!(SoundChangeAlias);
impl_deser_fromfullid!(SoundChangeAlias);

#[cfg(test)]
mod test {
    use crate::component::{FromFullIdentifier, SoundChangeAlias, ToFullIdentifier};

    #[test]
    fn test_valid_sound_change_aliases() {
        let cases = [
            "$sc:321",
            "$sc:123.ot_mt",
            "$sc:mt.ot_mt",
            "$sc:mt@derperor.ot_mt",
            "$sc:mt@123.ot_mt",
        ];
        for case in cases {
            let alias = SoundChangeAlias::from_full(case).unwrap();
            assert_eq!(alias.to_full(), case);
        }
    }

    #[test]
    fn test_invalid_sound_change_aliases() {
        let cases = [
            "$321",
            "$sc.321",
            "$sc:",
            "$sc:mt",
            "$sc:mt@derperor",
            "$sc:123@derperor.ot_mt",
            "$sc:mt.ot_mt.extra",
            "$mt.sc.ot_mt",
            "$mt@derperor.sc.ot_mt",
        ];
        for case in cases {
            assert!(SoundChangeAlias::from_full(case).is_err(), "{case}");
        }
    }
}
