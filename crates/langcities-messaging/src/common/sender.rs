use std::sync::mpsc::{SendError as MpscSendError, TrySendError as MpscTrySendError};

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

impl<T> From<MpscTrySendError<T>> for TrySendError<T> {
    fn from(value: MpscTrySendError<T>) -> Self {
        match value {
            MpscTrySendError::Full(v) => Self::Full(v),
            MpscTrySendError::Disconnected(v) => Self::Closed(v),
        }
    }
}

pub trait MessageSender<T: Message> {
    fn send(&self, value: T) -> impl Future<Output = Result<(), SendError<T>>>;
    fn try_send(&self, value: T) -> Result<(), TrySendError<T>>;
}
