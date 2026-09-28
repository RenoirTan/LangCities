use std::sync::mpsc::{SendError as MpscSendError, TrySendError as MpscTrySendError};

use async_trait::async_trait;

use crate::common::message::Message;

pub struct SendError<T>(pub T);

impl<T> From<MpscSendError<T>> for SendError<T> {
    fn from(value: MpscSendError<T>) -> Self {
        Self(value.0)
    }
}

pub enum TrySendError<T> {
    Full(T),
    Closed(T),
}

impl<T> From<SendError<T>> for TrySendError<T> {
    fn from(value: SendError<T>) -> Self {
        Self::Closed(value.0)
    }
}

impl<T> From<MpscTrySendError<T>> for TrySendError<T> {
    fn from(value: MpscTrySendError<T>) -> Self {
        match value {
            MpscTrySendError::Full(v) => Self::Full(v),
            MpscTrySendError::Disconnected(v) => Self::Closed(v),
        }
    }
}

#[async_trait]
pub trait MessageSender<T: Message> {
    async fn send(&self, value: T) -> Result<(), SendError<T>>;
    fn try_send(&self, value: T) -> Result<(), TrySendError<T>>;
}
