use std::sync::Arc;

use rayon::ThreadPool;

use crate::{
    manager::messaging::{DcManagerRxs, DcManagerTxs},
    message::event::{EntryFieldUpdateEvent, EntryFieldValueChanged},
    state::AppState,
    worker::messaging::DcWorkerMessaging,
};

#[derive(Clone)]
pub(crate) struct InnerDcManager {
    pub state: AppState,
    pub txs: DcManagerTxs,
    pub worker_msg: DcWorkerMessaging,
    pub thread_pool: Arc<ThreadPool>,
}

impl InnerDcManager {
    pub(crate) async fn do_efue(mut self, efue: EntryFieldUpdateEvent) {
        match &efue.changed {
            EntryFieldValueChanged::Override => self.do_efue_override(efue).await,
            EntryFieldValueChanged::Clean => self.do_efue_clean(efue).await,
        }
    }

    async fn do_efue_override(mut self, efue: EntryFieldUpdateEvent) {}

    async fn do_efue_clean(mut self, efue: EntryFieldUpdateEvent) {}
}

pub struct DcManager {
    pub(crate) inner: InnerDcManager,
    pub rxs: DcManagerRxs,
}

impl DcManager {
    pub(crate) fn new<A, R, T, W, P>(
        state: A,
        rxs: R,
        txs: T,
        worker_msg: W,
        thread_pool: ThreadPool,
    ) -> Self
    where
        A: Into<AppState>,
        R: Into<DcManagerRxs>,
        T: Into<DcManagerTxs>,
        W: Into<DcWorkerMessaging>,
        P: Into<ThreadPool>,
    {
        let (state, rxs, txs, worker_msg, thread_pool) = (
            state.into(),
            rxs.into(),
            txs.into(),
            worker_msg.into(),
            thread_pool.into(),
        );
        let inner = InnerDcManager {
            state,
            txs,
            worker_msg,
            thread_pool,
        };
        Self { inner, rxs }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                efue = self.rxs.efue_rx.recv() => {

                },
                _scue = self.rxs.scue_rx.recv() => {
                    unimplemented!();
                },
                _efde = self.rxs.efde_rx.recv() => {
                    unimplemented!();
                },
                _scde = self.rxs.scde_rx.recv() => {
                    unimplemented!();
                },
                _defouj = self.rxs.defouj_rx.recv() => {
                    unimplemented!();
                },
                _defcdj = self.rxs.defcdj_rx.recv() => {
                    unimplemented!();
                }
            }
        }
    }
}
