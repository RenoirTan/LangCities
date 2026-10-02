use chrono::Utc;

use crate::{
    dto::entry_field_jobs::CreateEntryFieldJobDto,
    error::{DcAppError, DcAppErrorTrait},
    hub::messaging::{HubRxs, HubTxs},
    message::{
        event::{EntryFieldUpdateEvent, EntryFieldValueChanged},
        job::NewEntryFieldCalculateDirtyJob,
    },
    repo::{entry_field_dependency::EntryFieldDependencyRepo, entry_field_job::EntryFieldJobRepo},
    state::AppState,
};

#[derive(Clone)]
pub(crate) struct InnerHub {
    pub state: AppState,
    pub txs: HubTxs,
}

impl InnerHub {
    pub(crate) async fn do_efue(self, efue: EntryFieldUpdateEvent) -> Result<(), DcAppError> {
        println!("Received EntryFieldUpdateEvent: {efue:#?}");
        return Ok(());
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

pub struct Hub {
    pub(crate) inner: InnerHub,
    pub rxs: HubRxs,
}

impl Hub {
    pub(crate) fn new<A, R, T>(state: A, rxs: R, txs: T) -> Self
    where
        A: Into<AppState>,
        R: Into<HubRxs>,
        T: Into<HubTxs>,
    {
        let (state, rxs, txs) = (state.into(), rxs.into(), txs.into());
        let inner = InnerHub { state, txs };
        Self { inner, rxs }
    }

    /// TODO: figure out what happens if end of rx occurs
    pub async fn run(mut self) {
        println!("Hi! I'm the hub");
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
