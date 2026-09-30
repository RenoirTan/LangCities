use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use sea_orm::{ActiveValue, Value};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    entity::entry_field_jobs,
    error::{DcAppError, DcAppErrorTrait},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum EntryFieldJobStatus {
    Queued,
    Working,
    Completed,
    Failed,
}

impl EntryFieldJobStatus {
    pub const fn active() -> &'static [Self] {
        &[Self::Queued, Self::Working]
    }

    pub const fn inactive() -> &'static [Self] {
        &[Self::Completed, Self::Failed]
    }

    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Queued | Self::Working)
    }

    pub const fn is_inactive(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }
}

impl FromStr for EntryFieldJobStatus {
    type Err = DcAppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match &s.to_uppercase()[..] {
            "QUEUED" => Self::Queued,
            "WORKING" => Self::Working,
            "COMPLETED" => Self::Completed,
            "FAILED" => Self::Failed,
            _ => {
                return Err(DcAppError::database(format!(
                    "Invalid entry field job status encountered! {}",
                    s
                )));
            }
        })
    }
}

impl Display for EntryFieldJobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Queued => f.write_str("QUEUED"),
            Self::Working => f.write_str("WORKING"),
            Self::Completed => f.write_str("COMPLETED"),
            Self::Failed => f.write_str("FAILED"),
        }
    }
}

impl Into<Value> for EntryFieldJobStatus {
    fn into(self) -> Value {
        Value::String(Some(self.to_string()))
    }
}

impl Into<Value> for &EntryFieldJobStatus {
    fn into(self) -> Value {
        (*self).into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct EntryFieldJobDto {
    pub id: i64,
    pub entry_field_id: i64,
    pub status: EntryFieldJobStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub n_attempts: i64,
}

impl TryFrom<entry_field_jobs::Model> for EntryFieldJobDto {
    type Error = DcAppError;

    fn try_from(value: entry_field_jobs::Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            entry_field_id: value.entry_field_id,
            status: value.status.parse::<EntryFieldJobStatus>()?,
            created_at: value.created_at,
            updated_at: value.updated_at,
            expires_at: value.expires_at,
            n_attempts: value.n_attempts,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateEntryFieldJobDto {
    pub entry_field_id: i64,
}

impl CreateEntryFieldJobDto {
    pub fn to_active_model(self, expires_at: DateTime<Utc>) -> entry_field_jobs::ActiveModel {
        entry_field_jobs::ActiveModel {
            entry_field_id: ActiveValue::Set(self.entry_field_id),
            expires_at: ActiveValue::Set(expires_at),
            status: ActiveValue::Set(EntryFieldJobStatus::Queued.to_string()),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateEntryFieldJobDto {
    pub status: EntryFieldJobStatus,
}

impl UpdateEntryFieldJobDto {
    pub fn to_active_model(self) -> entry_field_jobs::ActiveModel {
        let mut am = entry_field_jobs::ActiveModel::default();
        self.unchecked_update_active_model(&mut am);
        am
    }

    pub fn unchecked_update_active_model(
        self,
        model: &mut entry_field_jobs::ActiveModel,
    ) -> &mut entry_field_jobs::ActiveModel {
        model.status = ActiveValue::Set(self.status.to_string());
        model.updated_at = ActiveValue::Set(Utc::now());
        model
    }

    pub fn update_active_model(
        self,
        model: &mut entry_field_jobs::ActiveModel,
    ) -> Result<&mut entry_field_jobs::ActiveModel, DcAppError> {
        if let ActiveValue::Unchanged(original_status) = &model.status {
            let original_status = original_status.parse::<EntryFieldJobStatus>()?;
            if original_status.is_inactive() && self.status.is_active() {
                return Err(DcAppError::bad_state(format!(
                    "cannot reactivate inactive entry field job: {} -> {}",
                    original_status, self.status,
                )));
            }
        }
        model.status = ActiveValue::Set(self.status.to_string());
        model.updated_at = ActiveValue::Set(Utc::now());
        Ok(model)
    }
}
