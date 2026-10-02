use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::{Receiver, Sender};

use crate::message::job::{ToManagerJob, ToWorkerJob};

#[derive(Clone)]
pub struct WorkerMxs {
    pub to_manager: Sender<ToManagerJob>,
    pub to_worker: Arc<Mutex<Receiver<ToWorkerJob>>>,
}
