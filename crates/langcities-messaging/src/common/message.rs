use std::io::Read;

use chrono::{DateTime, Utc};
use serde::{Serialize, de::DeserializeOwned};

use crate::error::MessagingError;

pub trait BytesEncoder {
    fn to_bytes<T>(&mut self, data: T) -> Result<Vec<u8>, MessagingError>
    where
        T: Message;
}

pub trait BytesDecoder {
    fn from_read<R, T>(&mut self, read: R) -> Result<T, MessagingError>
    where
        R: Read,
        T: Message;

    fn from_bytes<T>(&mut self, bytes: &[u8]) -> Result<T, MessagingError>
    where
        T: Message,
    {
        self.from_read(bytes)
    }
}

pub trait Message: Serialize + DeserializeOwned {
    fn from_read<R, D>(read: R, decoder: &mut D) -> Result<Self, MessagingError>
    where
        R: Read,
        D: BytesDecoder,
    {
        decoder.from_read(read)
    }

    fn from_bytes<D>(bytes: &[u8], decoder: &mut D) -> Result<Self, MessagingError>
    where
        D: BytesDecoder,
    {
        decoder.from_bytes(bytes)
    }

    fn to_bytes<E>(self, encoder: &mut E) -> Result<Vec<u8>, MessagingError>
    where
        E: BytesEncoder,
    {
        encoder.to_bytes(self)
    }

    fn sent_at(&self) -> DateTime<Utc>;
}
