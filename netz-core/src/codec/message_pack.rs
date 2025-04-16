use std::{
    io::{Error, ErrorKind},
    marker::PhantomData,
};

use bytes::BytesMut;
use serde::{de::DeserializeOwned, Serialize};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug)]
pub struct MessagePack<O, I> {
    _marker: PhantomData<(O, I)>,
}

impl<O, I> Default for MessagePack<O, I>
where
    O: DeserializeOwned,
    I: Serialize,
{
    fn default() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<O, I> Decoder for MessagePack<O, I>
where
    O: DeserializeOwned,
{
    type Item = O;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        let item = rmp_serde::from_slice(src).map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        Ok(Some(item))
    }
}

impl<O, I> Encoder<I> for MessagePack<O, I>
where
    I: Serialize,
{
    type Error = Error;

    fn encode(&mut self, item: I, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let bytes = rmp_serde::to_vec(&item).map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        dst.extend_from_slice(&bytes);
        Ok(())
    }
}
