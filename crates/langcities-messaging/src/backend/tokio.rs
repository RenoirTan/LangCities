use tokio::sync::mpsc::{
    Receiver as MpscReceiver, Sender as MpscSender,
    error::{
        SendError as MpscSendError, TryRecvError as MpscTryRecvError,
        TrySendError as MpscTrySendError,
    },
};

use crate::common::{
    message::Message,
    receiver::{MessageReceiver, TryRecvError},
    sender::{MessageSender, SendError, TrySendError},
};

impl From<MpscTryRecvError> for TryRecvError {
    fn from(value: MpscTryRecvError) -> Self {
        match value {
            MpscTryRecvError::Empty => Self::Empty,
            MpscTryRecvError::Disconnected => Self::Disconnected,
        }
    }
}

impl<T> From<MpscSendError<T>> for SendError<T> {
    fn from(value: MpscSendError<T>) -> Self {
        Self(value.0)
    }
}

impl<T> From<MpscTrySendError<T>> for TrySendError<T> {
    fn from(value: MpscTrySendError<T>) -> Self {
        match value {
            MpscTrySendError::Full(v) => Self::Full(v),
            MpscTrySendError::Closed(v) => Self::Closed(v),
        }
    }
}

impl<T> MessageReceiver<T> for MpscReceiver<T>
where
    T: Message,
{
    async fn recv(&mut self) -> Option<T> {
        self.recv().await
    }

    async fn recv_many(&mut self, buffer: &mut Vec<T>, limit: usize) -> usize {
        self.recv_many(buffer, limit).await
    }

    fn try_recv(&mut self) -> Result<T, TryRecvError> {
        self.try_recv().map_err(TryRecvError::from)
    }
}

impl<T> MessageSender<T> for MpscSender<T>
where
    T: Message,
{
    async fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.send(value).await.map_err(SendError::from)
    }

    fn try_send(&self, value: T) -> Result<(), TrySendError<T>> {
        self.try_send(value).map_err(TrySendError::from)
    }
}
