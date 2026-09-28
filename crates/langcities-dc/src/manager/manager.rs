use rayon::ThreadPool;

use crate::{
    manager::messaging::DcManagerMessaging, state::AppState, worker::messaging::DcWorkerMessaging,
};

pub struct DcManager {
    pub state: AppState,
    pub msg: DcManagerMessaging,
    pub worker_msg: DcWorkerMessaging,
    pub thread_pool: ThreadPool,
}

impl DcManager {
    pub fn new<A, M, W, P>(state: A, msg: M, worker_msg: W, thread_pool: ThreadPool) -> Self
    where
        A: Into<AppState>,
        M: Into<DcManagerMessaging>,
        W: Into<DcWorkerMessaging>,
        P: Into<ThreadPool>,
    {
        let (state, msg, worker_msg, thread_pool) = (
            state.into(),
            msg.into(),
            worker_msg.into(),
            thread_pool.into(),
        );
        Self {
            state,
            msg,
            worker_msg,
            thread_pool,
        }
    }
}
