use chrono::{DateTime, Utc};
use langcities_lcdcdsl::component::Id;
use langcities_messaging::impl_message;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryFieldValueChanged {
    Override,
    Clean,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryFieldUpdateEvent {
    pub entry_field_id: Id,
    pub changed: EntryFieldValueChanged,
    pub message_at: DateTime<Utc>,
}
impl_message!(EntryFieldUpdateEvent);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoundChangeUpdateEvent {
    pub sound_change_id: Id,
    pub message_at: DateTime<Utc>,
}
impl_message!(SoundChangeUpdateEvent);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryFieldDeleteEvent {
    pub entry_field_id: Id,
    pub message_at: DateTime<Utc>,
}
impl_message!(EntryFieldDeleteEvent);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoundChangeDeleteEvent {
    pub sound_change_id: Id,
    pub message_at: DateTime<Utc>,
}
impl_message!(SoundChangeDeleteEvent);
