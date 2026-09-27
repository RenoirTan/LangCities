use chrono::{DateTime, Utc};
use langcities_lcdcdsl::component::Id;
use langcities_messaging::impl_message;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewEntryFieldOverrideUpdateJob {
    pub entry_field_id: Id,
    pub message_at: DateTime<Utc>,
}
impl_message!(NewEntryFieldOverrideUpdateJob);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoneEntryFieldOverrideUpdateJob {
    pub entry_field_id: Id,
    pub success: bool,
    pub message_at: DateTime<Utc>,
}
impl_message!(DoneEntryFieldOverrideUpdateJob);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewEntryFieldCalculateDirtyJob {
    pub entry_field_id: Id,
    pub message_at: DateTime<Utc>,
}
impl_message!(NewEntryFieldCalculateDirtyJob);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoneEntryFieldCalculateDirtyJob {
    pub entry_field_id: Id,
    pub success: bool,
    pub message_at: DateTime<Utc>,
}
impl_message!(DoneEntryFieldCalculateDirtyJob);
