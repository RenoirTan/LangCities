use std::sync::Arc;

use langcities_messaging::common::{receiver::MessageReceiver, sender::MessageSender};

use crate::message::{
    event::{
        EntryFieldDeleteEvent, EntryFieldUpdateEvent, SoundChangeDeleteEvent,
        SoundChangeUpdateEvent,
    },
    job::{
        DoneEntryFieldCalculateDirtyJob, DoneEntryFieldOverrideUpdateJob,
        NewEntryFieldCalculateDirtyJob, NewEntryFieldOverrideUpdateJob,
    },
};

pub struct ManagerRxs {
    pub efue_rx: Box<dyn MessageReceiver<EntryFieldUpdateEvent>>,
    pub scue_rx: Box<dyn MessageReceiver<SoundChangeUpdateEvent>>,
    pub efde_rx: Box<dyn MessageReceiver<EntryFieldDeleteEvent>>,
    pub scde_rx: Box<dyn MessageReceiver<SoundChangeDeleteEvent>>,
    pub defouj_rx: Box<dyn MessageReceiver<DoneEntryFieldOverrideUpdateJob>>,
    pub defcdj_rx: Box<dyn MessageReceiver<DoneEntryFieldCalculateDirtyJob>>,
}

#[derive(Clone)]
pub struct ManagerTxs {
    pub nefouj_tx: Arc<dyn MessageSender<NewEntryFieldOverrideUpdateJob>>,
    pub nefcdj_tx: Arc<dyn MessageSender<NewEntryFieldCalculateDirtyJob>>,
}
