use std::{
    collections::HashSet,
    task::{Context, Poll},
};

use tokio::{
    signal::windows::{
        CtrlBreak, CtrlC, CtrlClose, CtrlLogoff, CtrlShutdown, ctrl_break, ctrl_c, ctrl_close,
        ctrl_logoff, ctrl_shutdown,
    },
    sync::broadcast::{Receiver, WeakSender, channel},
    task::{JoinHandle, JoinSet},
};

use crate::lc::{SignalKind, Signaller};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum WindowsTokioSignalKind {
    CtrlBreak,
    CtrlC,
    CtrlClose,
    CtrlLogoff,
    CtrlShutdown,
}

impl WindowsTokioSignalKind {
    #[inline]
    pub fn to_signal(self) -> Result<WindowsTokioSignal, std::io::Error> {
        WindowsTokioSignal::from_kind(self)
    }
}

impl std::fmt::Display for WindowsTokioSignalKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

impl Into<SignalKind> for WindowsTokioSignalKind {
    fn into(self) -> SignalKind {
        match self {
            Self::CtrlBreak => SignalKind::Restart,
            _ => SignalKind::Stop,
        }
    }
}

pub enum WindowsTokioSignal {
    CtrlBreak(CtrlBreak),
    CtrlC(CtrlC),
    CtrlClose(CtrlClose),
    CtrlLogoff(CtrlLogoff),
    CtrlShutdown(CtrlShutdown),
}

impl WindowsTokioSignal {
    pub fn from_kind(kind: WindowsTokioSignalKind) -> Result<Self, std::io::Error> {
        Ok(match kind {
            WindowsTokioSignalKind::CtrlBreak => Self::CtrlBreak(ctrl_break()?),
            WindowsTokioSignalKind::CtrlC => Self::CtrlC(ctrl_c()?),
            WindowsTokioSignalKind::CtrlClose => Self::CtrlC(ctrl_close()?),
            WindowsTokioSignalKind::CtrlLogoff => Self::CtrlLogoff(ctrl_logoff()?),
            WindowsTokioSignalKind::CtrlShutdown => Self::CtrlShutdown(ctrl_shutdown()?),
        })
    }

    pub async fn recv(&mut self) -> Option<()> {
        match self {
            Self::CtrlBreak(c) => c.recv().await,
            Self::CtrlC(c) => c.recv().await,
            Self::CtrlClose(c) => c.recv().await,
            Self::CtrlLogoff(c) => c.recv().await,
            Self::CtrlShutdown(c) => c.recv().await,
        }
    }

    pub async fn recv_named(&mut self) -> WindowsTokioSignalKind {
        match self {
            Self::CtrlBreak(c) => {
                c.recv().await;
                WindowsTokioSignalKind::CtrlBreak
            }
            Self::CtrlC(c) => {
                c.recv().await;
                WindowsTokioSignalKind::CtrlC
            }
            Self::CtrlClose(c) => {
                c.recv().await;
                WindowsTokioSignalKind::CtrlClose
            }
            Self::CtrlLogoff(c) => {
                c.recv().await;
                WindowsTokioSignalKind::CtrlLogoff
            }
            Self::CtrlShutdown(c) => {
                c.recv().await;
                WindowsTokioSignalKind::CtrlShutdown
            }
        }
    }

    pub fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<()>> {
        match self {
            Self::CtrlBreak(c) => c.poll_recv(cx),
            Self::CtrlC(c) => c.poll_recv(cx),
            Self::CtrlClose(c) => c.poll_recv(cx),
            Self::CtrlLogoff(c) => c.poll_recv(cx),
            Self::CtrlShutdown(c) => c.poll_recv(cx),
        }
    }

    pub fn poll_recv_named(&mut self, cx: &mut Context<'_>) -> Poll<WindowsTokioSignalKind> {
        match self {
            Self::CtrlBreak(c) => c.poll_recv(cx).map(|_| WindowsTokioSignalKind::CtrlBreak),
            Self::CtrlC(c) => c.poll_recv(cx).map(|_| WindowsTokioSignalKind::CtrlC),
            Self::CtrlClose(c) => c.poll_recv(cx).map(|_| WindowsTokioSignalKind::CtrlClose),
            Self::CtrlLogoff(c) => c.poll_recv(cx).map(|_| WindowsTokioSignalKind::CtrlLogoff),
            Self::CtrlShutdown(c) => c
                .poll_recv(cx)
                .map(|_| WindowsTokioSignalKind::CtrlShutdown),
        }
    }
}

pub struct WindowsTokioSignaller {
    join_handle: JoinHandle<()>,
    /// [`std::sync::Weak`], so that drop `tx` when join_handle exits
    tx: WeakSender<WindowsTokioSignalKind>,
}

impl WindowsTokioSignaller {
    fn new<H, S>(join_handle: H, tx: S) -> Self
    where
        H: Into<JoinHandle<()>>,
        S: Into<WeakSender<WindowsTokioSignalKind>>,
    {
        let (join_handle, tx) = (join_handle.into(), tx.into());
        Self { join_handle, tx }
    }

    pub fn create<I>(kinds: I) -> Result<Self, std::io::Error>
    where
        I: IntoIterator<Item = WindowsTokioSignalKind>,
    {
        let kinds: HashSet<_> = kinds.into_iter().collect();
        let (tx, _rx) = channel(1);
        let weak_tx = tx.downgrade();
        let mut join_set = JoinSet::new();
        for kind in kinds {
            let mut inner_signal = kind.to_signal()?;
            join_set.spawn(async move {
                inner_signal.recv().await;
                (kind, inner_signal)
            });
        }
        let join_handle = tokio::spawn(async move {
            while let Some(Ok((kind, mut inner_signal))) = join_set.join_next().await {
                let _ = tx.send(kind);
                join_set.spawn(async move {
                    inner_signal.recv().await;
                    (kind, inner_signal)
                });
            }
        });
        Self::new(join_handle, weak_tx)
    }
}

impl Drop for WindowsTokioSignaller {
    #[inline]
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl Signaller for WindowsTokioSignaller {
    type Signal = WindowsTokioSignalKind;

    fn receiver(&self) -> Option<Receiver<WindowsTokioSignalKind>> {
        self.tx.upgrade().map(|strong| strong.subscribe())
    }

    fn shutdown(&mut self) {
        self.join_handle.abort();
    }
}
