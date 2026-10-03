use langcities_messaging::common::{receiver::MessageReceiver, sender::MessageSender};

use crate::message::job::{
    DoneEntryFieldCalculateDirtyJob, DoneEntryFieldOverrideUpdateJob,
    NewEntryFieldCalculateDirtyJob, NewEntryFieldOverrideUpdateJob,
};

pub struct ManagerRelayMxs {
    pub nefouj_rx: Box<dyn MessageReceiver<NewEntryFieldOverrideUpdateJob>>,
    pub defouj_tx: Box<dyn MessageSender<DoneEntryFieldOverrideUpdateJob>>,
    pub nefcdj_rx: Box<dyn MessageReceiver<NewEntryFieldCalculateDirtyJob>>,
    pub defcdj_tx: Box<dyn MessageSender<DoneEntryFieldCalculateDirtyJob>>,
}
