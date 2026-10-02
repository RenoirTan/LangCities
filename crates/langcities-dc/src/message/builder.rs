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
    api::messaging::{ApiMxs, DummyApiRxs},
    error::{DcAppError, DcAppErrorTrait},
    hub::messaging::{HubRxs, HubTxs},
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
    worker::messaging::WorkerMxs,
};

const DEFAULT_BOUNDED_MAX_BUFFER_SIZE: usize = 65536;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MxBackendKind {
    #[default]
    Tokio,
}

/// TODO: Add more configs to allow more backends
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MxBuilder {
    pub backend_kind: MxBackendKind,
    pub efue_max_buffer: Option<usize>,
    pub scue_max_buffer: Option<usize>,
    pub efde_max_buffer: Option<usize>,
    pub scde_max_buffer: Option<usize>,
    pub nefouj_max_buffer: Option<usize>,
    pub defouj_max_buffer: Option<usize>,
    pub nefcdj_max_buffer: Option<usize>,
    pub defcdj_max_buffer: Option<usize>,
}

impl MxBuilder {
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

    // TODO: support more backends
    pub fn build_only_api(self) -> Result<ApiMxs, DcAppError> {
        if matches!(self.backend_kind, MxBackendKind::Tokio) {
            return Err(DcAppError::failed_init(
                "cannot use tokio channels for api-only",
            ));
        }
        unreachable!();
    }

    // allows tokio channels that dump to dead senders
    // TODO: add more backends
    #[allow(unused)]
    pub(crate) fn build_dummy_api(self) -> Result<(ApiMxs, DummyApiRxs), DcAppError> {
        let (efue_tx, efue_rx) =
            make_paired_channels::<EntryFieldUpdateEvent>(self.efue_max_buffer);
        let (scue_tx, scue_rx) =
            make_paired_channels::<SoundChangeUpdateEvent>(self.scue_max_buffer);
        let (efde_tx, efde_rx) =
            make_paired_channels::<EntryFieldDeleteEvent>(self.efde_max_buffer);
        let (scde_tx, scde_rx) =
            make_paired_channels::<SoundChangeDeleteEvent>(self.scde_max_buffer);
        let api = ApiMxs {
            efue_tx: Arc::from(efue_tx),
            scue_tx: Arc::from(scue_tx),
            efde_tx: Arc::from(efde_tx),
            scde_tx: Arc::from(scde_tx),
        };
        let dummy = DummyApiRxs {
            efue_rx,
            scue_rx,
            efde_rx,
            scde_rx,
        };
        Ok((api, dummy))
    }

    pub fn build(self) -> Result<(ApiMxs, HubRxs, HubTxs, WorkerMxs), DcAppError> {
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
        let hub_rxs = HubRxs {
            efue_rx,
            scue_rx,
            efde_rx,
            scde_rx,
            defouj_rx,
            defcdj_rx,
        };
        let hub_txs = HubTxs {
            nefouj_tx: Arc::from(nefouj_tx),
            nefcdj_tx: Arc::from(nefcdj_tx),
        };
        let api = ApiMxs {
            efue_tx: Arc::from(efue_tx),
            scue_tx: Arc::from(scue_tx),
            efde_tx: Arc::from(efde_tx),
            scde_tx: Arc::from(scde_tx),
        };
        let worker = WorkerMxs {
            nefouj_rx: Arc::new(Mutex::new(nefouj_rx)),
            defouj_tx: Arc::from(defouj_tx),
            nefcdj_rx: Arc::new(Mutex::new(nefcdj_rx)),
            defcdj_tx: Arc::from(defcdj_tx),
        };
        Ok((api, hub_rxs, hub_txs, worker))
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
