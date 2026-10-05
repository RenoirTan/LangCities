use std::fmt::Display;

use futures::{Stream, StreamExt};
use tokio::sync::broadcast::Receiver;
use tokio_stream::wrappers::{BroadcastStream, errors::BroadcastStreamRecvError};

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

pub trait Signaller {
    type Signal: 'static + Into<SignalKind> + Clone + Send;

    fn receiver(&self) -> Option<Receiver<Self::Signal>>;
    fn shutdown(&mut self);
}
