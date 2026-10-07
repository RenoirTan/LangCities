use std::fmt::Display;

use futures::{Stream, StreamExt, future::ready};
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

pub trait Signaller {
    type Signal: 'static + Into<SignalKind> + Clone + Send;

    fn receiver(&self) -> Option<Receiver<Self::Signal>>;

    fn boxed_simple_stream(&self) -> Option<Box<dyn Stream<Item = SignalKind>>> {
        let rx = self.receiver()?;
        let stream = receiver_to_simple_stream(rx);
        let boxed = Box::new(stream);
        Some(boxed)
    }

    fn boxed_stream(
        &self,
    ) -> Option<Box<dyn Stream<Item = Result<SignalKind, BroadcastStreamRecvError>>>> {
        let rx = self.receiver()?;
        let stream = receiver_to_stream(rx);
        let boxed = Box::new(stream);
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
