use std::sync::Arc;

use langcities_messaging::common::{
    message::Message, receiver::MessageReceiver, sender::MessageSender,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{
    Mutex,
    mpsc::{channel, unbounded_channel},
};

use crate::{
    api::messaging::DcApiMessaging,
    error::DcAppError,
    message::{
        event::{
            EntryFieldDeleteEvent, EntryFieldUpdateEvent, SoundChangeDeleteEvent,
            SoundChangeUpdateEvent,
        },
        job::{
            DoneEntryFieldCalculateDirtyJob, DoneEntryFieldOverrideUpdateJob,
            NewEntryFieldCalculateDirtyJob, NewEntryFieldOverrideUpdateJob,
        },
    },
    worker::messaging::DcWorkerMessaging,
};

const DEFAULT_BOUNDED_MAX_BUFFER_SIZE: usize = 65536;

pub struct DcManagerRxs {
    pub efue_rx: Box<dyn MessageReceiver<EntryFieldUpdateEvent>>,
    pub scue_rx: Box<dyn MessageReceiver<SoundChangeUpdateEvent>>,
    pub efde_rx: Box<dyn MessageReceiver<EntryFieldDeleteEvent>>,
    pub scde_rx: Box<dyn MessageReceiver<SoundChangeDeleteEvent>>,
    pub defouj_rx: Box<dyn MessageReceiver<DoneEntryFieldOverrideUpdateJob>>,
    pub defcdj_rx: Box<dyn MessageReceiver<DoneEntryFieldCalculateDirtyJob>>,
}

#[derive(Clone)]
pub struct DcManagerTxs {
    pub nefouj_tx: Arc<dyn MessageSender<NewEntryFieldOverrideUpdateJob>>,
    pub nefcdj_tx: Arc<dyn MessageSender<NewEntryFieldCalculateDirtyJob>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DcManagerMessagingBackendKind {
    #[default]
    Tokio,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DcManagerMessagingBuilder {
    pub backend_kind: DcManagerMessagingBackendKind,
    pub efue_max_buffer: Option<usize>,
    pub scue_max_buffer: Option<usize>,
    pub efde_max_buffer: Option<usize>,
    pub scde_max_buffer: Option<usize>,
    pub nefouj_max_buffer: Option<usize>,
    pub defouj_max_buffer: Option<usize>,
    pub nefcdj_max_buffer: Option<usize>,
    pub defcdj_max_buffer: Option<usize>,
}

impl DcManagerMessagingBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_all_default_max_buffers(&mut self) -> &mut Self {
        self.with_all_max_buffers(Some(DEFAULT_BOUNDED_MAX_BUFFER_SIZE))
    }

    pub fn with_all_max_buffers(&mut self, size: Option<usize>) -> &mut Self {
        self.efue_max_buffer = size;
        self.scue_max_buffer = size;
        self.efde_max_buffer = size;
        self.scde_max_buffer = size;
        self.nefouj_max_buffer = size;
        self.defouj_max_buffer = size;
        self.nefcdj_max_buffer = size;
        self.defcdj_max_buffer = size;
        self
    }

    pub fn build(
        self,
    ) -> Result<
        (
            DcApiMessaging,
            DcManagerRxs,
            DcManagerTxs,
            DcWorkerMessaging,
        ),
        DcAppError,
    > {
        let (efue_tx, efue_rx) =
            make_paired_channels::<EntryFieldUpdateEvent>(self.efue_max_buffer);
        let (scue_tx, scue_rx) =
            make_paired_channels::<SoundChangeUpdateEvent>(self.scue_max_buffer);
        let (efde_tx, efde_rx) =
            make_paired_channels::<EntryFieldDeleteEvent>(self.efde_max_buffer);
        let (scde_tx, scde_rx) =
            make_paired_channels::<SoundChangeDeleteEvent>(self.scde_max_buffer);
        let (nefouj_tx, nefouj_rx) =
            make_paired_channels::<NewEntryFieldOverrideUpdateJob>(self.nefouj_max_buffer);
        let (defouj_tx, defouj_rx) =
            make_paired_channels::<DoneEntryFieldOverrideUpdateJob>(self.defouj_max_buffer);
        let (nefcdj_tx, nefcdj_rx) =
            make_paired_channels::<NewEntryFieldCalculateDirtyJob>(self.nefcdj_max_buffer);
        let (defcdj_tx, defcdj_rx) =
            make_paired_channels::<DoneEntryFieldCalculateDirtyJob>(self.defcdj_max_buffer);
        let manager_rxs = DcManagerRxs {
            efue_rx,
            scue_rx,
            efde_rx,
            scde_rx,
            defouj_rx,
            defcdj_rx,
        };
        let manager_txs = DcManagerTxs {
            nefouj_tx: Arc::from(nefouj_tx),
            nefcdj_tx: Arc::from(nefcdj_tx),
        };
        let api = DcApiMessaging {
            efue_tx: Arc::from(efue_tx),
            scue_tx: Arc::from(scue_tx),
            efde_tx: Arc::from(efde_tx),
            scde_tx: Arc::from(scde_tx),
        };
        let worker = DcWorkerMessaging {
            nefouj_rx: Arc::new(Mutex::new(nefouj_rx)),
            defouj_tx: Arc::from(defouj_tx),
            nefcdj_rx: Arc::new(Mutex::new(nefcdj_rx)),
            defcdj_tx: Arc::from(defcdj_tx),
        };
        Ok((api, manager_rxs, manager_txs, worker))
    }
}

fn make_paired_channels<T: Message + 'static>(
    bound: Option<usize>,
) -> (Box<dyn MessageSender<T>>, Box<dyn MessageReceiver<T>>) {
    match bound {
        Some(bound) => {
            let (tx, rx) = channel::<T>(bound);
            (Box::new(tx), Box::new(rx))
        }
        None => {
            let (tx, rx) = unbounded_channel::<T>();
            (Box::new(tx), Box::new(rx))
        }
    }
}
