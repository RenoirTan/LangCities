use langcities_lcdcdsl::component::Id;
use sea_orm::{ActiveValue, ColumnTrait, ExprTrait, sea_query::Expr};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    entity::entry_field_dependencies,
    error::{DcAppError, DcAppErrorTrait},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[non_exhaustive]
pub enum EntryFieldDependencyParentKind {
    EntryField,
    SoundChange,
}

impl TryFrom<i16> for EntryFieldDependencyParentKind {
    type Error = DcAppError;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::EntryField),
            2 => Ok(Self::SoundChange),
            _ => Err(DcAppError::bad_request(format!(
                "bad entry field dependency parent kind {value}"
            ))),
        }
    }
}

impl Into<i16> for EntryFieldDependencyParentKind {
    fn into(self) -> i16 {
        match self {
            Self::EntryField => 1,
            Self::SoundChange => 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct EntryFieldDependencyDto {
    pub child_entry_field_id: i64,
    pub parent_kind: EntryFieldDependencyParentKind,
    pub parent_id: i64,
}

impl TryFrom<entry_field_dependencies::Model> for EntryFieldDependencyDto {
    type Error = DcAppError;

    fn try_from(value: entry_field_dependencies::Model) -> Result<Self, Self::Error> {
        Ok(Self {
            child_entry_field_id: value.child_entry_field_id,
            parent_kind: value.parent_kind.try_into()?,
            parent_id: value.parent_id,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ParentEntryFieldDependencyDto {
    pub parent_kind: EntryFieldDependencyParentKind,
    pub parent_id: i64,
}

impl ParentEntryFieldDependencyDto {
    pub fn to_active_model(self, child_id: Id) -> entry_field_dependencies::ActiveModel {
        entry_field_dependencies::ActiveModel {
            child_entry_field_id: ActiveValue::Set(*child_id),
            parent_kind: ActiveValue::Set(self.parent_kind.into()),
            parent_id: ActiveValue::Set(self.parent_id),
        }
    }

    pub fn to_expr(self) -> Expr {
        let parent_kind: i16 = self.parent_kind.into();
        entry_field_dependencies::Column::ParentKind
            .eq(parent_kind)
            .and(entry_field_dependencies::Column::ParentId.eq(self.parent_id))
    }
}

pub type CreateEntryFieldDependencyDto = ParentEntryFieldDependencyDto;
pub type DeleteEntryFieldDependencyDto = ParentEntryFieldDependencyDto;
