use chrono::{DateTime, Utc};
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    entity::{entry_field_job_attempts, entry_field_jobs},
    error::{DcAppError, DcAppErrorTrait},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct EntryFieldJobAttemptDto {
    pub id: i64,
    /// parent job
    pub job_id: i64,
    /// idempotency field
    pub worker_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// if [`None`], allowed to complete/fail the attempt as long as not expired
    /// if [`Some`], not allowed to complete/fail the attempt even before expiry
    pub finished_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    /// if finished, then result is pending
    /// if finished and [`None`], then attempt failed
    /// if finished and [`Some`], then attempt completed
    pub calculated_value: Option<String>,
}

impl From<entry_field_job_attempts::Model> for EntryFieldJobAttemptDto {
    fn from(value: entry_field_job_attempts::Model) -> Self {
        Self {
            id: value.id,
            job_id: value.job_id,
            worker_id: value.worker_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            finished_at: value.finished_at,
            expires_at: value.expires_at,
            calculated_value: value.calculated_value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CreateEntryFieldJobAttemptDto {
    pub job_id: i64,
    pub worker_id: i64,
}

impl CreateEntryFieldJobAttemptDto {
    pub fn to_active_model(
        self,
        entry_field_job: entry_field_jobs::Model,
        expires_at: DateTime<Utc>,
    ) -> Result<entry_field_job_attempts::ActiveModel, DcAppError> {
        if entry_field_job.id != self.job_id {
            return Err(DcAppError::bad_state(format!(
                "mismatched entry field job ids: {} != {}",
                entry_field_job.id, self.job_id
            )));
        }
        let am = entry_field_job_attempts::ActiveModel {
            job_id: ActiveValue::Set(self.job_id),
            worker_id: ActiveValue::Set(self.worker_id),
            expires_at: ActiveValue::Set(expires_at),
            calculated_value: ActiveValue::Set(None),
            ..Default::default()
        };
        Ok(am)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct FailEntryFieldJobAttemptDto {}

impl FailEntryFieldJobAttemptDto {
    pub fn to_active_model(self) -> entry_field_job_attempts::ActiveModel {
        let mut am = entry_field_job_attempts::ActiveModel::default();
        self.unchecked_update_active_model(&mut am);
        am
    }

    fn unchecked_update_active_model(
        self,
        model: &mut entry_field_job_attempts::ActiveModel,
    ) -> &mut entry_field_job_attempts::ActiveModel {
        model.finished_at = ActiveValue::Set(Some(Utc::now()));
        model
    }

    pub fn update_active_model(
        self,
        model: &mut entry_field_job_attempts::ActiveModel,
    ) -> Result<&mut entry_field_job_attempts::ActiveModel, DcAppError> {
        if let ActiveValue::Unchanged(Some(finished_at)) = model.finished_at {
            return Err(DcAppError::bad_state(format!(
                "attempted to update finished entry field job attempt: {}",
                finished_at
            )));
        }
        Ok(self.unchecked_update_active_model(model))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct FinishEntryFieldJobAttemptDto {
    pub calculated_value: String,
}

impl FinishEntryFieldJobAttemptDto {
    pub fn to_active_model(self) -> entry_field_job_attempts::ActiveModel {
        let mut am = entry_field_job_attempts::ActiveModel::default();
        self.unchecked_update_active_model(&mut am);
        am
    }

    fn unchecked_update_active_model(
        self,
        model: &mut entry_field_job_attempts::ActiveModel,
    ) -> &mut entry_field_job_attempts::ActiveModel {
        model.finished_at = ActiveValue::Set(Some(Utc::now()));
        model.calculated_value = ActiveValue::Set(Some(self.calculated_value));
        model
    }

    pub fn update_active_model(
        self,
        model: &mut entry_field_job_attempts::ActiveModel,
    ) -> Result<&mut entry_field_job_attempts::ActiveModel, DcAppError> {
        if let ActiveValue::Unchanged(Some(finished_at)) = model.finished_at {
            return Err(DcAppError::bad_state(format!(
                "attempted to update finished entry field job attempt: {}",
                finished_at
            )));
        }
        Ok(self.unchecked_update_active_model(model))
    }
}
