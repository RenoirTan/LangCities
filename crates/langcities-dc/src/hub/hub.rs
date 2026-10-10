use chrono::Utc;

use crate::{
    dto::entry_field_jobs::CreateEntryFieldJobDto,
    error::{DcAppError, DcAppErrorTrait},
    hub::messaging::{HubRxs, HubTxs},
    message::{
        event::{EntryFieldUpdateEvent, EntryFieldValueChanged},
        job::{NewEntryFieldCalculateDirtyJob, NewEntryFieldOverrideUpdateJob},
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
        let job = repo.create_or_refresh_entry_field_job(&db, dto).await?;
        self.txs
            .nefouj_tx
            .send(NewEntryFieldOverrideUpdateJob {
                job_id: job.id.into(),
                message_at: Utc::now(),
            })
            .await
            .map_err(DcAppError::messaging)?;
        Ok(())
    }

    async fn do_efue_clean(self, efue: EntryFieldUpdateEvent) -> Result<(), DcAppError> {
        let dependency_repo = EntryFieldDependencyRepo::from_state(self.state.clone());
        let dependencies = dependency_repo
            .get_dependencies_of(&self.state.db, efue.entry_field_id)
            .await?;
        let dtos = dependencies
            .into_iter()
            .map(|dep| CreateEntryFieldJobDto {
                entry_field_id: dep.child_entry_field_id,
            })
            .collect::<Vec<_>>();
        let db = self.state.db.clone();
        let repo = EntryFieldJobRepo::from_state(self.state);
        let jobs = repo.create_or_refresh_entry_field_jobs(&db, dtos).await?;
        let msgs = jobs.into_iter().map(|job| NewEntryFieldCalculateDirtyJob {
            job_id: job.id.into(),
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
    pub async fn run(mut self, shutdown_signal: impl Future<Output = ()> + Send + 'static) {
        println!("Hi! I'm the hub");
        let mut shutdown_signal = tokio::spawn(shutdown_signal);
        let mut efue_rx_open = true;
        let mut scue_rx_open = true;
        let mut efde_rx_open = true;
        let mut scde_rx_open = true;
        let mut defouj_rx_open = true;
        let mut defcdj_rx_open = true;
        loop {
            tokio::select! {
                _ = &mut shutdown_signal => {
                    println!("Hub received shutdown signal!");
                    break;
                },
                efue = self.rxs.efue_rx.recv(), if efue_rx_open => {
                    if let Some(efue) = efue {
                        let _ = self.inner.clone().do_efue(efue).await;
                    } else {
                        println!("efue_rx closed");
                        efue_rx_open = false;
                    }
                },
                scue = self.rxs.scue_rx.recv(), if scue_rx_open => {
                    if let Some(scue) = scue {
                        println!("received {scue:#?}");
                    } else {
                        println!("scue_rx closed");
                        scue_rx_open = false;
                    }
                },
                efde = self.rxs.efde_rx.recv(), if efde_rx_open => {
                    if let Some(efde) = efde {
                        println!("received {efde:#?}")
                    } else {
                        println!("efde_rx closed");
                        efde_rx_open = false;
                    }
                },
                scde = self.rxs.scde_rx.recv(), if scde_rx_open => {
                    if let Some(scde) = scde {
                        println!("received {scde:#?}")
                    } else {
                        println!("scde_rx closed");
                        scde_rx_open = false;
                    }
                },
                defouj = self.rxs.defouj_rx.recv(), if defouj_rx_open => {
                    if let Some(defouj) = defouj {
                        println!("received {defouj:#?}");
                    } else {
                        println!("defouj_rx closed");
                        defouj_rx_open = false;
                    }
                },
                defcdj = self.rxs.defcdj_rx.recv(), if defcdj_rx_open => {
                    if let Some(defcdj) = defcdj {
                        println!("received {defcdj:#?}");
                    } else {
                        println!("defcdj_rx closed");
                        defcdj_rx_open = false;
                    }
                },
                else => break,
            }
        }
    }
}
