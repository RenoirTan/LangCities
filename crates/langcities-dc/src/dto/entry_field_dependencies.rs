use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::{DcAppError, DcAppErrorTrait};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[non_exhaustive]
pub enum EntryFieldDependencyParentKind {
    EntryField,
    SoundChange,
}

impl TryFrom<i64> for EntryFieldDependencyParentKind {
    type Error = DcAppError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::EntryField),
            2 => Ok(Self::SoundChange),
            _ => Err(DcAppError::bad_request(format!(
                "bad entry field dependency parent kind {value}"
            ))),
        }
    }
}

impl Into<i64> for EntryFieldDependencyParentKind {
    fn into(self) -> i64 {
        match self {
            Self::EntryField => 1,
            Self::SoundChange => 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct EntryFieldDependencyDto {
    pub child_entry_field_id: i64,
    pub parent_kind: i16,
    pub parent_id: i64,
}
