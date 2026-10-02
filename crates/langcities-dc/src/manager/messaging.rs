use langcities_messaging::common::{receiver::MessageReceiver, sender::MessageSender};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::message::job::{
    DoneEntryFieldCalculateDirtyJob, DoneEntryFieldOverrideUpdateJob,
    NewEntryFieldCalculateDirtyJob, NewEntryFieldOverrideUpdateJob, ToManagerJob, ToWorkerJob,
};

pub struct ManagerRelayMxs {
    pub nefouj_rx: Box<dyn MessageReceiver<NewEntryFieldOverrideUpdateJob>>,
    pub defouj_tx: Box<dyn MessageSender<DoneEntryFieldOverrideUpdateJob>>,
    pub nefcdj_rx: Box<dyn MessageReceiver<NewEntryFieldCalculateDirtyJob>>,
    pub defcdj_tx: Box<dyn MessageSender<DoneEntryFieldCalculateDirtyJob>>,
}

pub struct ManagerWorkerMxs {
    pub to_workers: Sender<ToWorkerJob>,
    pub to_manager: Receiver<ToManagerJob>,
}
