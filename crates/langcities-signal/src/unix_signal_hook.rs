use std::{borrow::Borrow, ffi::c_int};

use futures::stream::StreamExt;
use signal_hook::consts::signal::*;
use signal_hook_tokio::{Handle, Signals};
use tokio::{
    sync::broadcast::{Receiver, WeakSender, channel},
    task::JoinHandle,
};

use crate::lc::{SignalKind, Signaller};

impl From<c_int> for SignalKind {
    fn from(value: c_int) -> Self {
        match value {
            SIGALRM | SIGHUP | SIGINT | SIGPIPE | SIGQUIT | SIGTERM | SIGUSR2 => Self::Stop,
            SIGUSR1 => Self::Restart,
            _ => Self::Ignore,
        }
    }
}

pub struct UnixSignalHook {
    signal_handle: Handle,
    join_handle: JoinHandle<()>,
    tx: WeakSender<c_int>,
}

impl UnixSignalHook {
    fn new<S, J, T>(signal_handle: S, join_handle: J, tx: T) -> Self
    where
        S: Into<Handle>,
        J: Into<JoinHandle<()>>,
        T: Into<WeakSender<c_int>>,
    {
        let (signal_handle, join_handle, tx) =
            (signal_handle.into(), join_handle.into(), tx.into());
        Self {
            signal_handle,
            join_handle,
            tx,
        }
    }

    pub fn create<I, S>(kinds: I) -> Result<Self, std::io::Error>
    where
        I: IntoIterator<Item = S>,
        S: Borrow<c_int>,
    {
        let mut signals: Signals = Signals::new(kinds)?;
        let signal_handle = signals.handle();
        let (tx, _rx) = channel(1);
        let weak_tx = tx.downgrade();
        let join_handle = tokio::spawn(async move {
            while let Some(signal) = signals.next().await {
                let _ = tx.send(signal);
            }
        });
        Ok(Self::new(signal_handle, join_handle, weak_tx))
    }
}

impl Drop for UnixSignalHook {
    #[inline]
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl Signaller for UnixSignalHook {
    type Signal = c_int;

    fn receiver(&self) -> Option<Receiver<c_int>> {
        self.tx.upgrade().map(|strong| strong.subscribe())
    }

    fn shutdown(&mut self) {
        self.signal_handle.close();
        self.join_handle.abort();
    }
}
