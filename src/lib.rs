extern crate futures;
extern crate bytes;
extern crate serde;
extern crate rmp_serde;
extern crate tokio_serde;

use futures::{Stream, Poll, Sink, StartSend};
use bytes::{Bytes, BytesMut, Buf, IntoBuf};
use serde::{Serialize, Deserialize};
use tokio_serde::{Serializer, Deserializer, FramedRead, FramedWrite};

use std::marker::PhantomData;

use std::io::Error;

struct MsgPack<T> {
    ghost: PhantomData<T>,
}

fn into_io_error<U, E>(res: Result<U,E>) -> Result<U, std::io::Error>
where
    E: Into<Box<std::error::Error + Send + Sync>>,
{
    res.map_err(|err| {
        std::io::Error::new(std::io::ErrorKind::Other, err)
    })
}

impl<T> Deserializer<T> for MsgPack<T>
    where for <'a> T: Deserialize<'a>,
{
    type Error = std::io::Error;

    fn deserialize(&mut self, src: &Bytes) -> Result<T, Self::Error> {
        into_io_error(rmp_serde::decode::from_read(src.into_buf().reader()))
    }
}

impl<T: Serialize> Serializer<T> for MsgPack<T> {
    type Error = std::io::Error;

    fn serialize(&mut self, item: &T) -> Result<BytesMut, Self::Error> {
        into_io_error(rmp_serde::encode::to_vec(item).map(Into::into))
    }
}

pub struct ReadMsgPack<T,U> {
    inner: FramedRead<T, U, MsgPack<U>>,
}

impl<T, U> ReadMsgPack<T, U>
    where T: Stream,
          T::Error: From<std::io::Error>,
          for<'a> U: Deserialize<'a>,
          Bytes: From<T::Item>,
{
    pub fn new(inner: T) -> ReadMsgPack<T, U> {
        let mp = MsgPack { ghost: PhantomData };
        ReadMsgPack { inner: FramedRead::new(inner, mp) }
    }
}

impl<T, U> ReadMsgPack<T, U> {
    pub fn get_ref(&self) -> &T {
        self.inner.get_ref()
    }
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
}

impl<T, U> Stream for ReadMsgPack<T, U>
    where T: Stream,
          T::Error: From<std::io::Error>,
          for<'a> U: Deserialize<'a>,
          Bytes: From<T::Item>,
{
    type Item = U;
    type Error = T::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.inner.poll()
    }
}

impl<T, U> Sink for ReadMsgPack<T, U>
    where T: Sink,
{
    type SinkItem = T::SinkItem;
    type SinkError = T::SinkError;

    fn start_send(&mut self, item: T::SinkItem)
                  -> StartSend<T::SinkItem, T::SinkError> {
        self.get_mut().start_send(item)
    }

    fn poll_complete(&mut self) -> Poll<(), T::SinkError> {
        self.get_mut().poll_complete()
    }

    fn close(&mut self) -> Poll<(), T::SinkError> {
        self.get_mut().close()
    }
}

pub struct WriteMsgPack<T: Sink, U> {
    inner: FramedWrite<T, U, MsgPack<U>>,
}

impl<T, U> WriteMsgPack<T, U>
    where T: Sink<SinkItem = BytesMut>,
          T::SinkError: From<std::io::Error>,
          U: Serialize,
{
    pub fn new(inner: T) -> WriteMsgPack<T, U> {
        let mp = MsgPack { ghost: PhantomData };
        let fw = FramedWrite::new(inner, mp);
        WriteMsgPack { inner: fw }
    }
}

impl<T: Sink, U> WriteMsgPack<T, U> {
    pub fn get_ref(&self) -> &T {
        self.inner.get_ref()
    }
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
}

impl<T, U> Stream for WriteMsgPack<T, U>
    where T: Stream + Sink,
{
    type Item = T::Item;
    type Error = T::Error;

    fn poll(&mut self) -> Poll<Option<T::Item>, T::Error> {
        self.get_mut().poll()
    }
}

impl<T, U> Sink for WriteMsgPack<T, U>
    where T: Sink<SinkItem = BytesMut>,
          T::SinkError: From<std::io::Error>,
          U: Serialize,
{
    type SinkItem = U;
    type SinkError = T::SinkError;

    fn start_send(&mut self, item: U) -> StartSend<U, T::SinkError> {
        self.inner.start_send(item)
    }

    fn poll_complete(&mut self) -> Poll<(), T::SinkError> {
        self.inner.poll_complete()
    }

    fn close(&mut self) -> Poll<(), T::SinkError> {
        self.inner.poll_complete()
    }
}