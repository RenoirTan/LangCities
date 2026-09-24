use std::{borrow::Cow, fmt::Display, str::FromStr};

use crate::{
    component::{Id, Slug, VernacularAlias},
    error::{DslError, DslErrorKind, DslErrorTrait},
    impl_deser_fromstr, impl_ser_display,
};

/// $<vernacular_alias>.(<entry_index>/_).<field_slug>
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EfDepFullFormat {
    pub vernacular_alias: VernacularAlias,
    pub entry_index: Option<Id>,
    pub field_slug: Slug,
}

impl EfDepFullFormat {
    fn fmt_entry_index(&self) -> Cow<'static, str> {
        match &self.entry_index {
            Some(idx) => Cow::Owned(idx.to_string()),
            None => Cow::Borrowed("_"),
        }
    }

    fn from_parts(va: &str, ei: &str, fs: &str) -> Result<Self, DslError> {
        let vernacular_alias = va.parse::<VernacularAlias>()?;
        let entry_index = if ei == "_" {
            None
        } else {
            Some(ei.parse::<Id>()?)
        };
        let field_slug = fs.parse::<Slug>()?;
        Ok(Self {
            vernacular_alias,
            entry_index,
            field_slug,
        })
    }
}

impl Display for EfDepFullFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "${}.{}.{}",
            self.vernacular_alias,
            self.fmt_entry_index(),
            self.field_slug
        )
    }
}

impl FromStr for EfDepFullFormat {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let stripped = s
            .strip_prefix('$')
            .ok_or_else(|| DslError::bad_value_of(s))?;
        let parts: Vec<_> = stripped.rsplitn(3, '.').collect();
        if parts.len() < 3 {
            return Err(DslError::bad_value_of(s));
        }
        Self::from_parts(parts[2], parts[1], parts[0])
    }
}

impl_ser_display!(EfDepFullFormat);
impl_deser_fromstr!(EfDepFullFormat);

/// $<entry_id>.<field_slug>
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EfDepEsFormat {
    pub entry_id: Id,
    pub field_slug: Slug,
}

impl EfDepEsFormat {
    fn from_parts(ei: &str, fs: &str) -> Result<Self, DslError> {
        let entry_id = ei.parse::<Id>()?;
        let field_slug = fs.parse::<Slug>()?;
        Ok(Self {
            entry_id,
            field_slug,
        })
    }
}

impl Display for EfDepEsFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}.{}", self.entry_id, self.field_slug)
    }
}

impl FromStr for EfDepEsFormat {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let e = || DslError::bad_value_of(s);
        let stripped = s.strip_prefix('$').ok_or_else(e)?;
        let mut parts = stripped.splitn(2, ".");
        let ei = parts.next().ok_or_else(e)?;
        let fs = parts.next().ok_or_else(e)?;
        Self::from_parts(ei, fs)
    }
}

impl_ser_display!(EfDepEsFormat);
impl_deser_fromstr!(EfDepEsFormat);

/// $ef.<field_id>
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EfDepIdFormat {
    pub field_id: Id,
}

impl EfDepIdFormat {
    fn from_parts(fi: &str) -> Result<Self, DslError> {
        let field_id = fi.parse::<Id>()?;
        Ok(Self { field_id })
    }
}

impl Display for EfDepIdFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "$ef.{}", self.field_id)
    }
}

impl FromStr for EfDepIdFormat {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let stripped = s
            .strip_prefix("$ef.")
            .ok_or_else(|| DslError::bad_value_of(s))?;
        Self::from_parts(stripped)
    }
}

impl_ser_display!(EfDepIdFormat);
impl_deser_fromstr!(EfDepIdFormat);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EfDepAlias {
    Full(EfDepFullFormat),
    Es(EfDepEsFormat),
    Id(EfDepIdFormat),
}

impl Display for EfDepAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full(a) => a.fmt(f),
            Self::Es(a) => a.fmt(f),
            Self::Id(a) => a.fmt(f),
        }
    }
}

impl FromStr for EfDepAlias {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let stripped = s
            .strip_prefix('$')
            .ok_or_else(|| DslError::bad_value_of(s))?;
        let parts: Vec<_> = stripped.rsplitn(3, '.').collect();
        if parts.len() == 3 {
            let a = EfDepFullFormat::from_parts(parts[2], parts[1], parts[0])?;
            Ok(Self::Full(a))
        } else if parts.len() == 2 {
            if parts[1] == "ef" {
                Ok(Self::Id(EfDepIdFormat::from_parts(parts[0])?))
            } else {
                Ok(Self::Es(EfDepEsFormat::from_parts(parts[1], parts[0])?))
            }
        } else {
            Err(DslError::bad_value_of(s))
        }
    }
}

impl_ser_display!(EfDepAlias);
impl_deser_fromstr!(EfDepAlias);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ScDepFullFormat {
    pub vernacular_alias: VernacularAlias,
    pub sc_slug: Slug,
}

impl ScDepFullFormat {
    fn from_parts(va: &str, ss: &str) -> Result<Self, DslError> {
        let vernacular_alias = va.parse::<VernacularAlias>()?;
        let sc_slug = ss.parse::<Slug>()?;
        Ok(Self {
            vernacular_alias,
            sc_slug,
        })
    }
}

impl Display for ScDepFullFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}.sc.{}", self.vernacular_alias, self.sc_slug)
    }
}

impl FromStr for ScDepFullFormat {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let e = || DslError::bad_value_of(s);
        let stripped = s.strip_prefix('$').ok_or_else(e)?;
        let parts: Vec<_> = stripped.rsplitn(3, '.').collect();
        if parts.len() < 3 || parts[1] != "sc" {
            return Err(e());
        }
        Self::from_parts(parts[2], parts[0])
    }
}

impl_ser_display!(ScDepFullFormat);
impl_deser_fromstr!(ScDepFullFormat);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ScDepIdFormat {
    pub sc_id: Id,
}

impl ScDepIdFormat {
    fn from_parts(si: &str) -> Result<Self, DslError> {
        let sc_id = si.parse::<Id>()?;
        Ok(Self { sc_id })
    }
}

impl Display for ScDepIdFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "$sc.{}", self.sc_id)
    }
}

impl FromStr for ScDepIdFormat {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let stripped = s
            .strip_prefix("$sc.")
            .ok_or_else(|| DslError::bad_value_of(s))?;
        Self::from_parts(stripped)
    }
}

impl_ser_display!(ScDepIdFormat);
impl_deser_fromstr!(ScDepIdFormat);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ScDepAlias {
    Full(ScDepFullFormat),
    Id(ScDepIdFormat),
}

impl Display for ScDepAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full(a) => a.fmt(f),
            Self::Id(a) => a.fmt(f),
        }
    }
}

impl FromStr for ScDepAlias {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let stripped = s
            .strip_prefix('$')
            .ok_or_else(|| DslError::bad_value_of(s))?;
        let parts: Vec<_> = stripped.rsplitn(3, '.').collect();
        if parts.len() >= 3 && parts[1] == "sc" {
            Ok(Self::Full(ScDepFullFormat::from_parts(parts[2], parts[0])?))
        } else if parts.len() == 2 && parts[1] == "sc" {
            Ok(Self::Id(ScDepIdFormat::from_parts(parts[0])?))
        } else {
            Err(DslError::bad_value_of(s))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DepAlias {
    Ef(EfDepAlias),
    Sc(ScDepAlias),
}

impl Display for DepAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ef(a) => a.fmt(f),
            Self::Sc(a) => a.fmt(f),
        }
    }
}

impl FromStr for DepAlias {
    type Err = DslError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // only retries if error is because first case does not match
        s.parse::<EfDepAlias>()
            .map(Self::Ef)
            .or_else(|e| match e.kind {
                DslErrorKind::BadValue => s.parse::<ScDepAlias>().map(Self::Sc),
                _ => Err(e),
            })
    }
}

#[cfg(test)]
mod test {
    use crate::dependency::{DepAlias, EfDepAlias, ScDepAlias};

    #[test]
    fn test_valid_efdep() {
        let cases = [
            "$mt.123.ipa",
            "$mt@user._.transliteration",
            "$456.definition",
            "$ef.789",
        ];
        for case in cases {
            let result = case.parse::<EfDepAlias>().unwrap().to_string();
            assert_eq!(result, case);
        }
    }

    #[test]
    fn test_invalid_efdep() {
        let cases = [
            "$123.456",
            "$712",
            "mt.123.ipa",
            "mt@user._.transliteration",
            "456.definition",
            "ef.789",
        ];
        for case in cases {
            assert!(case.parse::<EfDepAlias>().is_err());
        }
    }

    #[test]
    fn test_valid_scdep() {
        let cases = ["$mt.sc.ot_mt", "$mt@user.sc.ot_mt", "$sc.123"];
        for case in cases {
            let result = case.parse::<ScDepAlias>().unwrap().to_string();
            assert_eq!(result, case);
        }
    }

    #[test]
    fn test_invalid_scdep() {
        let cases = [
            "mt.sc.ot_mt",
            "mt@user.sc.ot_mt",
            "sc.123",
            "$mt.bad.slug",
            "$123",
            "$123.456",
        ];
        for case in cases {
            assert!(case.parse::<ScDepAlias>().is_err());
        }
    }

    #[test]
    fn test_valid_depalias() {
        let cases = [
            "$mt.123.ipa",
            "$mt@user._.transliteration",
            "$456.definition",
            "$ef.789",
            "$mt.sc.ot_mt",
            "$mt@user.sc.ot_mt",
            "$sc.123",
        ];
        for case in cases {
            let result = case.parse::<DepAlias>().unwrap().to_string();
            assert_eq!(result, case);
        }
    }

    #[test]
    fn test_invalid_depalias() {
        let cases = [
            "mt.123.ipa",
            "mt@user._.transliteration",
            "456.definition",
            "ef.789",
            "mt.sc.ot_mt",
            "mt@user.sc.ot_mt",
            "sc.123",
            "$mt.123.5",
            "$1@user._.transliteration",
            "$_.definition",
            "$ar.789",
            "$mt.bad.ot_mt",
            "$mt@user.ouch.ot_mt",
            "$sc.me",
        ];
        for case in cases {
            assert!(case.parse::<DepAlias>().is_err());
        }
    }
}
