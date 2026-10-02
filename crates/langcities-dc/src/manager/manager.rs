use chrono::Utc;

use crate::{
    dto::entry_field_jobs::CreateEntryFieldJobDto,
    error::{DcAppError, DcAppErrorTrait},
    manager::messaging::{ManagerRxs, ManagerTxs},
    message::{
        event::{EntryFieldUpdateEvent, EntryFieldValueChanged},
        job::NewEntryFieldCalculateDirtyJob,
    },
    repo::{entry_field_dependency::EntryFieldDependencyRepo, entry_field_job::EntryFieldJobRepo},
    state::AppState,
    worker::messaging::WorkerMxs,
};

#[derive(Clone)]
pub(crate) struct InnerManager {
    pub state: AppState,
    pub txs: ManagerTxs,
    pub worker_mxs: WorkerMxs,
}

impl InnerManager {
    pub(crate) async fn do_efue(self, efue: EntryFieldUpdateEvent) -> Result<(), DcAppError> {
        match &efue.changed {
            EntryFieldValueChanged::Override => self.do_efue_override(efue).await,
            EntryFieldValueChanged::Clean => self.do_efue_clean(efue).await,
        }
    }

    /// create job to get new
    async fn do_efue_override(self, efue: EntryFieldUpdateEvent) -> Result<(), DcAppError> {
        let dto = CreateEntryFieldJobDto {
            entry_field_id: *efue.entry_field_id,
        };
        let db = self.state.db.clone();
        let repo = EntryFieldJobRepo::from_state(self.state);
        repo.create_or_refresh_entry_field_job(&db, dto).await?;
        Ok(())
    }

    async fn do_efue_clean(self, efue: EntryFieldUpdateEvent) -> Result<(), DcAppError> {
        let dependency_repo = EntryFieldDependencyRepo::from_state(self.state.clone());
        let dependencies = dependency_repo
            .get_dependencies_of(&self.state.db, efue.entry_field_id)
            .await?;
        let msgs = dependencies
            .into_iter()
            .map(|d| NewEntryFieldCalculateDirtyJob {
                entry_field_id: d.child_entry_field_id.into(),
                message_at: Utc::now(),
            });
        for msg in msgs {
            self.txs
                .nefcdj_tx
                .send(msg)
                .await
                .map_err(DcAppError::other)?;
        }
        Ok(())
    }
}

pub struct Manager {
    pub(crate) inner: InnerManager,
    pub rxs: ManagerRxs,
}

impl Manager {
    pub(crate) fn new<A, R, T, W>(state: A, rxs: R, txs: T, worker_mxs: W) -> Self
    where
        A: Into<AppState>,
        R: Into<ManagerRxs>,
        T: Into<ManagerTxs>,
        W: Into<WorkerMxs>,
    {
        let (state, rxs, txs, worker_mxs) =
            (state.into(), rxs.into(), txs.into(), worker_mxs.into());
        let inner = InnerManager {
            state,
            txs,
            worker_mxs,
        };
        Self { inner, rxs }
    }

    /// TODO: figure out what happens if end of rx occurs
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                efue = self.rxs.efue_rx.recv() => {
                    if let Some(efue) = efue {
                        let _ = self.inner.clone().do_efue(efue).await;
                    } else {
                        return;
                    }
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
