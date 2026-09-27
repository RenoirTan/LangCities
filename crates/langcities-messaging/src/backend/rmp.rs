use std::io::Read;

use rmp_serde::{Deserializer, Serializer, config::DefaultConfig, decode::ReadReader};
use serde::Deserialize;

use crate::{
    common::message::{BytesDecoder, BytesEncoder, Message},
    error::{MessagingError, MessagingErrorTrait},
};

pub struct MessagePacker;

impl MessagePacker {
    pub fn new() -> Self {
        Self
    }

    fn new_serializer(&self) -> Serializer<Vec<u8>, DefaultConfig> {
        Serializer::new(vec![])
    }

    #[inline]
    fn inner_encode<T>(&mut self, data: &T) -> Result<Vec<u8>, MessagingError>
    where
        T: Message,
    {
        let mut serializer = self.new_serializer();
        data.serialize(&mut serializer)
            .map_err(MessagingError::serialization)?;
        Ok(serializer.into_inner())
    }

    pub fn encode<T>(&mut self, data: &T) -> Result<Vec<u8>, MessagingError>
    where
        T: Message,
    {
        self.inner_encode(data)
    }
}

impl BytesEncoder for MessagePacker {
    fn to_bytes<T>(&mut self, data: &T) -> Result<Vec<u8>, MessagingError>
    where
        T: Message,
    {
        self.inner_encode(data)
    }
}

pub struct MessageUnpacker;

impl MessageUnpacker {
    pub fn new() -> Self {
        Self
    }

    pub fn new_deserializer<R>(&self, read: R) -> Deserializer<ReadReader<R>, DefaultConfig>
    where
        R: Read,
    {
        Deserializer::new(read)
    }

    #[inline]
    fn inner_decode<R, T>(&mut self, read: R) -> Result<T, MessagingError>
    where
        R: Read,
        T: Message,
    {
        let mut deserializer = self.new_deserializer(read);
        Deserialize::deserialize(&mut deserializer).map_err(MessagingError::deserialization)
    }

    pub fn decode<R, T>(&mut self, read: R) -> Result<T, MessagingError>
    where
        R: Read,
        T: Message,
    {
        self.inner_decode(read)
    }
}

impl BytesDecoder for MessageUnpacker {
    fn from_read<R, T>(&mut self, read: R) -> Result<T, MessagingError>
    where
        R: Read,
        T: Message,
    {
        self.inner_decode(read)
    }
}
