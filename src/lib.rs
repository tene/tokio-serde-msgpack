extern crate bytes;
extern crate futures_core;
extern crate futures_sink;
extern crate rmp_serde;
extern crate serde;
extern crate tokio_io;

use bytes::{BufMut, BytesMut};
use rmp_serde::Deserializer;
use serde::{Deserialize, Serialize};
use tokio_io::codec::{Decoder, Encoder, FramedRead, FramedWrite};
use tokio_io::{AsyncRead, AsyncWrite};

use std::io;
use std::marker::PhantomData;

pub type MsgPackReader<'lt, T, R> =
    tokio_io::codec::FramedRead<tokio_io::io::ReadHalf<T>, MsgPackDecoder<'lt, R>>;
pub type MsgPackWriter<T, S> =
    tokio_io::codec::FramedWrite<tokio_io::io::WriteHalf<T>, MsgPackEncoder<S>>;

pub fn from_io<'lt, T, R, S>(io: T) -> (MsgPackReader<'lt, T, R>, MsgPackWriter<T, S>)
where
    T: AsyncRead + AsyncWrite,
    R: Deserialize<'lt>,
    S: Serialize,
{
    let (rx, tx) = io.split();
    let rx2 = FramedRead::new(rx, MsgPackDecoder::<'lt, R>::new());
    let tx2 = FramedWrite::new(tx, MsgPackEncoder::<S>::new());
    (rx2, tx2)
}

pub struct MsgPackDecoder<'de, T: 'de>
where
    T: Deserialize<'de>,
{
    ghost: PhantomData<&'de T>,
}

impl<'de, T> MsgPackDecoder<'de, T>
where
    T: Deserialize<'de>,
{
    pub fn new() -> Self {
        MsgPackDecoder { ghost: PhantomData }
    }
}

impl<'de, T> Decoder for MsgPackDecoder<'de, T>
where
    T: Deserialize<'de>,
{
    type Item = T;
    type Error = io::Error;
    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<T>, io::Error> {
        Deserialize::deserialize(&mut Deserializer::new(&buf[..]))
            .map_err(|_e| io::Error::from(io::ErrorKind::InvalidData))
    }
}

pub struct MsgPackEncoder<T>
where
    T: Serialize,
{
    ghost: PhantomData<T>,
}

impl<T> MsgPackEncoder<T>
where
    T: Serialize,
{
    pub fn new() -> Self {
        MsgPackEncoder { ghost: PhantomData }
    }
}

impl<T> Encoder for MsgPackEncoder<T>
where
    T: Serialize,
{
    type Item = T;
    type Error = io::Error;
    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        rmp_serde::to_vec(&item)
            .map(|buf| {
                dst.reserve(buf.len());
                dst.put(buf);
            })
            .map_err(|_e| io::Error::from(io::ErrorKind::InvalidData))
    }
}

pub struct MsgPackCodec<'de, R: 'de, S>
where
    R: Deserialize<'de>,
    S: Serialize,
{
    ghost: PhantomData<(&'de R, S)>,
}

impl<'de, R, S> Encoder for MsgPackCodec<'de, R, S>
where
    R: Deserialize<'de>,
    S: Serialize,
{
    type Item = S;
    type Error = io::Error;
    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut enc = MsgPackEncoder { ghost: PhantomData };
        enc.encode(item, dst)
    }
}

impl<'de, R, S> Decoder for MsgPackCodec<'de, R, S>
where
    R: Deserialize<'de>,
    S: Serialize,
{
    type Item = R;
    type Error = io::Error;
    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<R>, io::Error> {
        let mut dec = MsgPackDecoder { ghost: PhantomData };
        dec.decode(buf)
    }
}

/*
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

    fn poll_next(&mut self, cx: &mut Context) -> Poll<Option<Self::Item>, Self::Error> {
        self.inner.poll_next(cx)
    }
}

impl<T, U> Sink for ReadMsgPack<T, U>
    where T: Sink,
{
    type SinkItem = T::SinkItem;
    type SinkError = T::SinkError;

    fn start_send(&mut self, item: T::SinkItem)
                  -> Result<(), T::SinkError> {
        self.get_mut().start_send(item)
    }

    fn poll_close(&mut self, cx: &mut Context) -> Poll<(), T::SinkError> {
        self.get_mut().poll_close(cx)
    }
    fn poll_ready(&mut self, cx: &mut Context) -> Poll<(), T::SinkError> {
        self.get_mut().poll_ready(cx)
    }
    fn poll_flush(&mut self, cx: &mut Context) -> Poll<(), T::SinkError> {
        self.get_mut().poll_flush(cx)
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

    fn poll_next(&mut self, cx: &mut Context) -> Poll<Option<Self::Item>, Self::Error> {
        self.inner.poll_next(cx)
    }
}

impl<T, U> Sink for WriteMsgPack<T, U>
    where T: Sink<SinkItem = BytesMut>,
          T::SinkError: From<std::io::Error>,
          U: Serialize,
{
    type SinkItem = U;
    type SinkError = T::SinkError;

    fn start_send(&mut self, item: T::SinkItem)
                  -> Result<(), T::SinkError> {
        self.get_mut().start_send(item)
    }

    fn poll_close(&mut self, cx: &mut Context) -> Poll<(), T::SinkError> {
        self.get_mut().poll_close(cx)
    }
    fn poll_ready(&mut self, cx: &mut Context) -> Poll<(), T::SinkError> {
        self.get_mut().poll_ready(cx)
    }
    fn poll_flush(&mut self, cx: &mut Context) -> Poll<(), T::SinkError> {
        self.get_mut().poll_flush(cx)
    }
}
*/
