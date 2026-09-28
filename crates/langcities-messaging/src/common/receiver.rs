use std::sync::mpsc::TryRecvError as MpscTryRecvError;

use async_trait::async_trait;

use crate::common::message::Message;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TryRecvError {
    Empty,
    Disconnected,
}

impl From<MpscTryRecvError> for TryRecvError {
    fn from(value: MpscTryRecvError) -> Self {
        match value {
            MpscTryRecvError::Empty => Self::Empty,
            MpscTryRecvError::Disconnected => Self::Disconnected,
        }
    }
}

/// Based on [`tokio::sync::mpsc::Receiver`]
#[async_trait]
pub trait MessageReceiver<T: Message> {
    async fn recv(&mut self) -> Option<T>;

    async fn recv_many(&mut self, buffer: &mut Vec<T>, limit: usize) -> usize {
        let mut i: usize = 0;
        while i < limit {
            if let Some(message) = self.recv().await {
                buffer.push(message);
            } else {
                break;
            }
            i += 1;
        }
        i
    }

    fn try_recv(&mut self) -> Result<T, TryRecvError>;

    /// [`None`] if connection closed and no messages added to the buffer
    /// Returns [`Some`] upon first empty. `Some(0)` is a possible return value.
    /// If connection is closed after at least one message has been added, this function must return
    /// [`Some`]
    fn try_recv_many(&mut self, buffer: &mut Vec<T>, limit: usize) -> Option<usize> {
        let mut i: usize = 0;
        while i < limit {
            match self.try_recv() {
                Ok(message) => buffer.push(message),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    if i == 0 {
                        return None;
                    } else {
                        break;
                    }
                }
            }
            i += 1;
        }
        Some(i)
    }
}
