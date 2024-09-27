//! Raw data straight from the source

use crate::connect::JdwpTransport;
use crate::raw::codec::RawCodec;
use crate::raw::packet::{AnyRawPacket, RawCommandPacket};
use futures::Sink;
use futures::Stream;
use pin_project::pin_project;
use std::io;
use std::io::Error;
use std::marker::PhantomData;
use std::pin::{pin, Pin};
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::{unbounded_channel, Receiver, UnboundedReceiver};
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{error_span, instrument, trace};

pub mod codec;
pub mod packet;

/// A raw packet sink
#[derive(Debug)]
#[pin_project]
pub struct RawPacketSink<I: AsyncWrite + Unpin>(#[pin] FramedWrite<I, RawCodec>);

impl<I: AsyncWrite + Unpin> Sink<RawCommandPacket> for RawPacketSink<I> {
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::<RawCommandPacket>::poll_ready(self.project().0, cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: RawCommandPacket) -> Result<(), Self::Error> {
        Sink::<RawCommandPacket>::start_send(self.project().0, item)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::<RawCommandPacket>::poll_flush(self.project().0, cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Sink::<RawCommandPacket>::poll_close(self.project().0, cx)
    }
}

/// A raw packet stream
#[derive(Debug)]
#[pin_project]
pub struct RawPacketStream<O> {
    sender: UnboundedReceiver<Result<AnyRawPacket, Error>>,
    task: JoinHandle<()>,
    _phantom: PhantomData<O>,
}

impl<O> Stream for RawPacketStream<O> {
    type Item = Result<AnyRawPacket, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().sender.poll_recv(cx)
    }
}
/// The raw client
#[derive(Debug)]
pub struct RawJdwpClient<T: JdwpTransport> {
    sink: FramedWrite<T::Output, RawCodec>,
    sender: UnboundedReceiver<Result<AnyRawPacket, Error>>,
    task: JoinHandle<()>,
}

impl<T> RawJdwpClient<T>
where
    T: JdwpTransport,
{
    /// Creates a new RawJdwpClient
    pub fn new(input: T::Input, output: T::Output) -> Self {
        let codec = RawCodec::default();
        let raw_sink = FramedWrite::new(output, codec);
        let mut raw_stream = FramedRead::new(input, codec);

        let (tx, rx) = unbounded_channel::<Result<AnyRawPacket, Error>>();

        let task = tokio::spawn(async move {
            let span = error_span!("packet-recv-loop");
            let _guard = span.enter();
            while let Some(packet) = raw_stream.next().await {
                tx.send(packet).unwrap();
            }
        });

        RawJdwpClient {
            sink: raw_sink,
            sender: rx,
            task,
        }
    }

    /// Splits into the sink and the stream
    pub fn into_split(self) -> (RawPacketStream<T::Input>, RawPacketSink<T::Output>) {
        let Self { sink, sender, task } = self;
        let sink = sink;
        let sender = sender;
        (
            RawPacketStream {
                sender,
                task,
                _phantom: PhantomData,
            },
            RawPacketSink(sink),
        )
    }
}
