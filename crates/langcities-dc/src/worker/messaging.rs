use std::sync::Arc;

use langcities_messaging::common::{receiver::MessageReceiver, sender::MessageSender};
use tokio::sync::Mutex;

use crate::message::job::{
    DoneEntryFieldCalculateDirtyJob, DoneEntryFieldOverrideUpdateJob,
    NewEntryFieldCalculateDirtyJob, NewEntryFieldOverrideUpdateJob,
};

#[derive(Clone)]
pub struct DcWorkerMessaging {
    pub nefouj_rx: Arc<Mutex<Box<dyn MessageReceiver<NewEntryFieldOverrideUpdateJob>>>>,
    pub defouj_tx: Arc<dyn MessageSender<DoneEntryFieldOverrideUpdateJob>>,
    pub nefcdj_rx: Arc<Mutex<Box<dyn MessageReceiver<NewEntryFieldCalculateDirtyJob>>>>,
    pub defcdj_tx: Arc<dyn MessageSender<DoneEntryFieldCalculateDirtyJob>>,
}
