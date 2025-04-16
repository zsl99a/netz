use std::{
    io::Error,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::{Bytes, BytesMut};
use futures::{Sink, Stream};
use serde::{de::DeserializeOwned, Serialize};
use tokio_util::codec::{Decoder, Encoder};

pin_project_lite::pin_project! {
    pub struct IoSerdeFramed<S, O, I, F> {
        #[pin]
        stream: S,
        codec: F,
        _marker: PhantomData<(O, I)>,
    }
}

impl<S, O, I, F> IoSerdeFramed<S, O, I, F>
where
    S: Stream<Item = Result<BytesMut, Error>> + Sink<Bytes, Error = Error>,
    F: Decoder<Item = O, Error = Error> + Encoder<I, Error = Error>,
    O: DeserializeOwned,
    I: Serialize,
{
    pub fn new(stream: S, codec: F) -> Self {
        Self {
            stream,
            codec,
            _marker: PhantomData,
        }
    }
}

impl<S, O, I, F> Stream for IoSerdeFramed<S, O, I, F>
where
    S: Stream<Item = Result<BytesMut, Error>> + Sink<Bytes, Error = Error>,
    F: Decoder<Item = O, Error = Error> + Encoder<I, Error = Error>,
    O: DeserializeOwned,
    I: Serialize,
{
    type Item = Result<O, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut project = self.project();
        match project.stream.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(mut bytes))) => {
                let item = project.codec.decode(&mut bytes);
                match item {
                    Ok(Some(item)) => Poll::Ready(Some(Ok(item))),
                    Ok(None) => Poll::Pending,
                    Err(error) => Poll::Ready(Some(Err(error))),
                }
            }
            Poll::Ready(Some(Err(error))) => Poll::Ready(Some(Err(error))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<S, O, I, F> Sink<I> for IoSerdeFramed<S, O, I, F>
where
    S: Stream<Item = Result<BytesMut, Error>> + Sink<Bytes, Error = Error>,
    F: Decoder<Item = O, Error = Error> + Encoder<I, Error = Error>,
    O: DeserializeOwned,
    I: Serialize,
{
    type Error = <S as Sink<Bytes>>::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().stream.as_mut().poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), Self::Error> {
        let mut dst = BytesMut::new();
        let mut project = self.project();
        project.codec.encode(item, &mut dst)?;
        project.stream.as_mut().start_send(dst.freeze())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().stream.as_mut().poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().stream.as_mut().poll_close(cx)
    }
}
