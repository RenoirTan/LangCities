use std::collections::VecDeque;

use tokio::{
    runtime::{Builder, Runtime},
    sync::mpsc::{Receiver, Sender, channel},
    task::{JoinError, JoinSet},
};

use crate::{
    error::{DcAppError, DcAppErrorTrait},
    manager::messaging::ManagerRelayMxs,
    message::job::{ToManagerJob, ToWorkerJob},
    state::AppState,
};

pub struct Manager {
    #[allow(unused)]
    state: AppState,
    relay_mxs: Option<ManagerRelayMxs>,
    w_tx: Option<Sender<ToWorkerJob>>,
    w_rx: Option<Receiver<ToWorkerJob>>,
    m_tx: Option<Sender<ToManagerJob>>,
    m_rx: Option<Receiver<ToManagerJob>>,
    runtime: Option<Runtime>,
}

impl Manager {
    pub fn new<S, RM, RT>(state: S, relay_mxs: RM, runtime: RT) -> Self
    where
        S: Into<AppState>,
        RM: Into<ManagerRelayMxs>,
        RT: Into<Runtime>,
    {
        let (state, relay_mxs, runtime) =
            (state.into(), Some(relay_mxs.into()), Some(runtime.into()));
        // TODO: use state to configure channel size
        let (w_tx, w_rx) = channel(65536);
        let (m_tx, m_rx) = channel(65536);
        Self {
            state,
            relay_mxs,
            w_tx: Some(w_tx),
            w_rx: Some(w_rx),
            m_tx: Some(m_tx),
            m_rx: Some(m_rx),
            runtime,
        }
    }

    // TODO: update and use config
    pub fn create<S, RM>(state: S, relay_mxs: RM) -> Result<Self, DcAppError>
    where
        S: Into<AppState>,
        RM: Into<ManagerRelayMxs>,
    {
        let runtime = Builder::new_multi_thread()
            .worker_threads(8)
            .enable_all()
            .build()
            .map_err(DcAppError::failed_init)?;
        Ok(Self::new(state, relay_mxs, runtime))
    }

    pub async fn run(mut self) -> Result<(), DcAppError> {
        let Some(runtime) = &self.runtime else {
            return Err(DcAppError::failed_init("manager async runtime is missing"));
        };
        let (Some(relay_mxs), Some(w_tx), Some(w_rx), Some(m_tx), Some(m_rx)) = (
            self.relay_mxs.take(),
            self.w_tx.take(),
            self.w_rx.take(),
            self.m_tx.take(),
            self.m_rx.take(),
        ) else {
            return Err(DcAppError::failed_init("manager channels consumed"));
        };
        let result = runtime
            .spawn(async move {
                let mut join_set = JoinSet::<()>::new();
                join_set.spawn(Self::run_relay(relay_mxs, w_tx, m_rx));
                join_set.spawn(Self::run_inner(w_rx, m_tx, 8));
                let mut join_errors = Vec::<JoinError>::new();
                while let Some(result) = join_set.join_next().await {
                    if let Err(je) = result {
                        join_errors.push(je);
                    }
                }
                join_errors
            })
            .await;
        match result {
            Ok(errors) => {
                if errors.len() <= 0 {
                    Ok(())
                } else {
                    Err(DcAppError::other(
                        "did not successfully join manager threads",
                    ))
                }
            }
            Err(_je) => Err(DcAppError::other(
                "did not successfully join manager threads",
            )),
        }
    }

    async fn run_relay(
        mut relay_mxs: ManagerRelayMxs,
        w_tx: Sender<ToWorkerJob>,
        mut m_rx: Receiver<ToManagerJob>,
    ) {
        let mut nefouj_open = true;
        let mut nefcdj_open = true;
        let mut defouj_open = true;
        let mut defcdj_open = true;
        // if w_tx.is_closed || m_rx.is_closed,
        // that means run_inner has stopped
        loop {
            tokio::select! {
                nefouj = relay_mxs.nefouj_rx.recv(), if nefouj_open && defouj_open && !w_tx.is_closed() => {
                    let Some(nefouj) = nefouj else {
                        nefouj_open = false;
                        println!("nefouj_rx closed!");
                        continue;
                    };
                    let w_job = ToWorkerJob::Nefouj(nefouj);
                    let _ = w_tx.send(w_job).await;
                },
                nefcdj = relay_mxs.nefcdj_rx.recv(), if nefcdj_open && defcdj_open && !w_tx.is_closed() => {
                    let Some(nefcdj) = nefcdj else {
                        nefcdj_open = false;
                        println!("nefcdj_rx closed!");
                        continue;
                    };
                    let w_job = ToWorkerJob::Nefcdj(nefcdj);
                    let _ = w_tx.send(w_job).await;
                },
                m_job = m_rx.recv(), if !m_rx.is_closed() && (defouj_open || defcdj_open) => {
                    let Some(m_job) = m_job else {
                        println!("m_rx closed!");
                        continue;
                    };
                    match m_job {
                        ToManagerJob::Defouj(defouj) => {
                            if let Err(_e) = relay_mxs
                                .defouj_tx
                                .send(defouj)
                                .await
                            {
                                defouj_open = false;
                                println!("defouj_tx closed");
                            }
                        }
                        ToManagerJob::Defcdj(defcdj) => {
                            if let Err(_e) = relay_mxs
                                .defcdj_tx
                                .send(defcdj)
                                .await
                            {
                                defcdj_open = false;
                                println!("defcdj_tx closed");
                            }
                        }
                    };
                },
                else => break,
            }
        }
        println!("Exiting Manager::run_relay");
    }

    async fn run_inner(
        mut w_rx: Receiver<ToWorkerJob>,
        m_tx: Sender<ToManagerJob>,
        max_jobs: usize,
    ) {
        let mut join_set = JoinSet::<Result<ToManagerJob, DcAppError>>::new();
        let mut job_queue = VecDeque::<ToWorkerJob>::new();
        loop {
            tokio::select! {
                // don't receive if workers are full
                w_job = w_rx.recv(), if !w_rx.is_closed() && join_set.len() < max_jobs => {
                    let Some(w_job) = w_job else {
                        println!("w_rx closed!");
                        continue;
                    };
                    Self::handle_wjob(w_job, &mut join_set, &mut job_queue).await;
                },
                done = join_set.join_next(), if !m_tx.is_closed() && !join_set.is_empty() => {
                    let Some(done) = done else {
                        println!("m_tx closed!");
                        continue;
                    };
                    match done {
                        Ok(Ok(msg)) => {
                            let _ = m_tx.send(msg).await;
                        },
                        Ok(Err(e)) => {
                            println!("{e} occurred");
                        },
                        Err(je) => {
                            println!("{je} occurred");
                        }
                    };
                },
                else => break,
            };
        }
        // clean up
        while let Some(done) = join_set.join_next().await {
            match done {
                Ok(Ok(msg)) => {
                    let _ = m_tx.send(msg).await;
                }
                Ok(Err(e)) => {
                    println!("{e} occurred");
                }
                Err(je) => {
                    println!("{je} occurred");
                }
            };
        }
        println!("Exiting Manager::run_inner");
    }

    async fn handle_wjob(
        w_job: ToWorkerJob,
        join_set: &mut JoinSet<Result<ToManagerJob, DcAppError>>,
        job_queue: &mut VecDeque<ToWorkerJob>,
    ) {
        let w_job = if let Some(job) = job_queue.pop_front() {
            job_queue.push_back(w_job);
            job
        } else {
            w_job
        };
        join_set.spawn(async move {
            let _ = w_job;
            Err(DcAppError::other("unimplemented"))
        });
    }
}

impl Drop for Manager {
    fn drop(&mut self) {
        self.relay_mxs.take();
        self.w_tx.take();
        self.w_rx.take();
        self.m_tx.take();
        self.m_rx.take();
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
}
