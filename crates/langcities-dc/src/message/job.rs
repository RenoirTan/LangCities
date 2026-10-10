use chrono::{DateTime, Utc};
use langcities_lcdcdsl::component::Id;
use langcities_messaging::impl_message;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewEntryFieldOverrideUpdateJob {
    pub job_id: Id,
    pub message_at: DateTime<Utc>,
}
impl_message!(NewEntryFieldOverrideUpdateJob);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoneEntryFieldOverrideUpdateJob {
    pub job_attempt_id: Id,
    pub message_at: DateTime<Utc>,
}
impl_message!(DoneEntryFieldOverrideUpdateJob);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewEntryFieldCalculateDirtyJob {
    pub job_id: Id,
    pub message_at: DateTime<Utc>,
}
impl_message!(NewEntryFieldCalculateDirtyJob);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoneEntryFieldCalculateDirtyJob {
    pub job_attempt_id: Id,
    pub message_at: DateTime<Utc>,
}
impl_message!(DoneEntryFieldCalculateDirtyJob);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToWorkerJob {
    Nefouj(NewEntryFieldOverrideUpdateJob),
    Nefcdj(NewEntryFieldCalculateDirtyJob),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToManagerJob {
    Defouj(DoneEntryFieldOverrideUpdateJob),
    Defcdj(DoneEntryFieldCalculateDirtyJob),
}
