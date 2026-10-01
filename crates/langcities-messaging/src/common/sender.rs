use std::{
    error::Error as StdError,
    fmt::{Debug, Display},
    sync::mpsc::{SendError as MpscSendError, TrySendError as MpscTrySendError},
};

use async_trait::async_trait;

use crate::common::message::Message;

pub struct SendError<T>(pub T);

impl<T> From<MpscSendError<T>> for SendError<T> {
    fn from(value: MpscSendError<T>) -> Self {
        Self(value.0)
    }
}

impl<T> Debug for SendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl<T> Display for SendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SendError")
    }
}

impl<T> StdError for SendError<T> {}

pub enum TrySendError<T> {
    Full(T),
    // TODO: rename to Disconnected
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

impl<T> Debug for TrySendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl<T> Display for TrySendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full(_) => f.write_str("TrySendError::Full"),
            Self::Closed(_) => f.write_str("TrySendError::Closed"),
        }
    }
}

impl<T> StdError for TrySendError<T> {}

#[async_trait]
pub trait MessageSender<T: Message>: Send + Sync {
    async fn send(&self, value: T) -> Result<(), SendError<T>>;
    fn try_send(&self, value: T) -> Result<(), TrySendError<T>>;
}
