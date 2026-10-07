use std::collections::HashSet;

use tokio::{
    signal::unix::{SignalKind, signal},
    sync::broadcast::{Receiver, WeakSender, channel},
    task::{JoinHandle, JoinSet},
};

use crate::lc::{SignalKind as LcSignalKind, Signaller};

const SIGALRM: SignalKind = SignalKind::alarm();
const SIGHUP: SignalKind = SignalKind::hangup();
const SIGINT: SignalKind = SignalKind::interrupt();
const SIGPIPE: SignalKind = SignalKind::pipe();
const SIGQUIT: SignalKind = SignalKind::quit();
const SIGTERM: SignalKind = SignalKind::terminate();
const SIGUSR1: SignalKind = SignalKind::user_defined1();
const SIGUSR2: SignalKind = SignalKind::user_defined2();
const NON_IGNORED_SIGS: [SignalKind; 8] = [
    SIGALRM, SIGHUP, SIGINT, SIGPIPE, SIGQUIT, SIGTERM, SIGUSR1, SIGUSR2,
];

impl From<SignalKind> for LcSignalKind {
    fn from(value: SignalKind) -> Self {
        match value {
            SIGALRM | SIGHUP | SIGINT | SIGPIPE | SIGQUIT | SIGTERM | SIGUSR2 => Self::Stop,
            SIGUSR1 => Self::Restart,
            _ => Self::Ignore,
        }
    }
}

pub struct UnixTokioSignaller {
    join_handle: JoinHandle<()>,
    /// [`std::sync::Weak`], so that drop `tx` when join_handle exits
    tx: WeakSender<SignalKind>,
}

impl UnixTokioSignaller {
    fn new<H, S>(join_handle: H, tx: S) -> Self
    where
        H: Into<JoinHandle<()>>,
        S: Into<WeakSender<SignalKind>>,
    {
        let (join_handle, tx) = (join_handle.into(), tx.into());
        Self { join_handle, tx }
    }

    pub fn create<I>(kinds: I) -> Result<Self, std::io::Error>
    where
        I: IntoIterator<Item = SignalKind>,
    {
        let kinds: HashSet<_> = kinds.into_iter().collect();
        let (tx, _rx) = channel(1);
        let weak_tx = tx.downgrade();
        let mut join_set = JoinSet::new();
        for kind in kinds {
            let mut inner_signal = signal(kind)?;
            join_set.spawn(async move {
                inner_signal.recv().await;
                (kind, inner_signal)
            });
        }
        let join_handle = tokio::spawn(async move {
            while let Some(Ok((kind, mut inner_signal))) = join_set.join_next().await {
                // ignore error, that just means no thread has requested a receiver yet
                let _ = tx.send(kind);
                join_set.spawn(async move {
                    inner_signal.recv().await;
                    (kind, inner_signal)
                });
            }
            // when tx gets dropped, Receivers will all get RecvError::Closed
            // will be treated as SIGINT
        });
        Ok(Self::new(join_handle, weak_tx))
    }

    pub fn create_default() -> Result<Self, std::io::Error> {
        Self::create(NON_IGNORED_SIGS)
    }
}

impl Drop for UnixTokioSignaller {
    #[inline]
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl Signaller for UnixTokioSignaller {
    type Signal = SignalKind;

    fn receiver(&self) -> Option<Receiver<Self::Signal>> {
        self.tx.upgrade().map(|strong| strong.subscribe())
    }

    fn shutdown(&mut self) {
        self.join_handle.abort();
    }
}
