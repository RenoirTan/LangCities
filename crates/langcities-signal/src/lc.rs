use std::{fmt::Display, pin::Pin};

use futures::{Stream, StreamExt, future::ready};
use tokio::sync::broadcast::Receiver;
use tokio_stream::wrappers::{BroadcastStream, errors::BroadcastStreamRecvError};

#[cfg(all(unix, feature = "signal-hook"))]
use crate::unix_signal_hook::UnixSignalHook;
#[cfg(unix)]
use crate::unix_tokio::UnixTokioSignaller;
#[cfg(windows)]
use crate::windows_tokio::WindowsTokioSignaller;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SignalKind {
    Stop,
    Restart,
    Ignore,
}

impl Display for SignalKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

pub fn receiver_to_stream<T>(
    rx: Receiver<T>,
) -> impl Stream<Item = Result<SignalKind, BroadcastStreamRecvError>>
where
    T: 'static + Into<SignalKind> + Clone + Send,
{
    BroadcastStream::new(rx).map(|r| r.map(|t| t.into()))
}

pub fn receiver_to_simple_stream<T>(rx: Receiver<T>) -> impl Stream<Item = SignalKind>
where
    T: 'static + Into<SignalKind> + Clone + Send,
{
    BroadcastStream::new(rx).filter_map(|r| {
        ready(match r {
            Ok(s) => match s.into() {
                SignalKind::Ignore => None,
                s => Some(s),
            },
            Err(_) => None,
        })
    })
}

pub type BoxedSignalStream =
    Pin<Box<dyn Stream<Item = Result<SignalKind, BroadcastStreamRecvError>>>>;
pub type BoxedSimpleSignalStream = Pin<Box<dyn Stream<Item = SignalKind>>>;

pub trait Signaller {
    type Signal: 'static + Into<SignalKind> + Clone + Send;

    fn receiver(&self) -> Option<Receiver<Self::Signal>>;

    fn boxed_simple_stream(&self) -> Option<BoxedSimpleSignalStream> {
        let rx = self.receiver()?;
        let stream = receiver_to_simple_stream(rx);
        let boxed = Box::pin(stream);
        Some(boxed)
    }

    fn boxed_stream(&self) -> Option<BoxedSignalStream> {
        let rx = self.receiver()?;
        let stream = receiver_to_stream(rx);
        let boxed = Box::pin(stream);
        Some(boxed)
    }

    fn shutdown(&mut self);
}

pub trait SignallerExt: Signaller {
    fn stream(&self) -> Option<impl Stream<Item = Result<SignalKind, BroadcastStreamRecvError>>> {
        self.receiver().map(|rx| receiver_to_stream(rx))
    }

    fn simple_stream(&self) -> Option<impl Stream<Item = SignalKind>> {
        self.receiver().map(|rx| receiver_to_simple_stream(rx))
    }
}

impl<T> SignallerExt for T where T: Signaller {}

/// dyn-compatible [`Signaller`] that has no generic parameters
pub trait SafeSignaller {
    fn boxed_simple_stream(&self) -> Option<BoxedSimpleSignalStream>;
    fn boxed_stream(&self) -> Option<BoxedSignalStream>;
    fn shutdown(&mut self);
}

impl<S> SafeSignaller for S
where
    S: Signaller,
{
    fn boxed_simple_stream(&self) -> Option<BoxedSimpleSignalStream> {
        self.boxed_simple_stream()
    }

    fn boxed_stream(&self) -> Option<BoxedSignalStream> {
        self.boxed_stream()
    }

    fn shutdown(&mut self) {
        self.shutdown();
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SignallerKind {
    #[cfg(unix)]
    UnixTokio,
    #[cfg(all(unix, feature = "signal-hook"))]
    UnixSignalHook,
    #[cfg(windows)]
    WindowsTokio,
}

impl Default for SignallerKind {
    fn default() -> Self {
        #[cfg(all(unix, not(feature = "signal-hook")))]
        return Self::UnixTokio;

        #[cfg(all(unix, feature = "signal-hook"))]
        return Self::UnixSignalHook;

        #[cfg(windows)]
        return Self::WindowsTokio;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SignallerConfig {
    kind: SignallerKind,
}

impl SignallerConfig {
    pub fn new<K>(kind: K) -> Self
    where
        K: Into<SignallerKind>,
    {
        let kind = kind.into();
        Self { kind }
    }

    pub fn create_signaller(self) -> Result<Box<dyn SafeSignaller>, std::io::Error> {
        let boxed = match self.kind {
            #[cfg(unix)]
            SignallerKind::UnixTokio => Box::new(UnixTokioSignaller::create_default()?),
            #[cfg(all(unix, feature = "signal-hook"))]
            SignallerKind::UnixSignalHook => Box::new(UnixSignalHook::create_default()?),
            #[cfg(windows)]
            WindowsTokio => Box::new(WindowsTokioSignaller::create_default()?),
        };
        Ok(boxed)
    }
}
