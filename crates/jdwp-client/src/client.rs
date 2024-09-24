use crate::codec::{JdwpCodec, JdwpDecoder};
use crate::events::{to_events, EventHandler, Events, NotAnEventError};
use crate::events::{Event, OwnedEventHandler};
use crate::id_sizes::IdSizes;
use crate::packet::JdwpCommand;
use crate::raw::codec::RawCodec;
use crate::raw::packet::{AnyRawPacket, RawCommandPacket, RawReplyPacket};
use crate::raw::{RawJdwpClient, RawPacketSink};
use bytes::BytesMut;
use futures_util::task::SpawnExt;
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::io;
use std::io::{Error, ErrorKind};
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio::sync::{Mutex, RwLock};
use tokio::task::{JoinHandle, JoinSet};
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{error, error_span, instrument, trace, warn, Instrument};

use tokio::sync::oneshot::Receiver as OneshotReceiver;
use tokio::sync::oneshot::Sender as OneshotSender;

static JDWP_HANDSHAKE: &[u8; 14] = b"JDWP-Handshake";

/// A non-blocking jdwp client
pub struct JdwpClient {
    id_sizes: IdSizes,
    tasks: JoinSet<()>,
    event_handlers: Arc<RwLock<Vec<OwnedEventHandler<Error>>>>,
    raw_packet_sink: Mutex<RawPacketSink>,
    next_id: AtomicU32,
    codec: Arc<RwLock<JdwpCodec>>,
    one_shots: Arc<RwLock<HashMap<u32, OneshotSender<RawReplyPacket>>>>,
}

impl JdwpClient {
    /// Creates a new jdwp client over a tcp stream
    pub async fn create(stream: TcpStream) -> io::Result<Self> {
        let (input, output) = stream.into_split();
        create_client(input, output).await
    }

    pub async fn on_event<E: EventHandler<Err = io::Error> + Sync>(&mut self, event_handler: E) {
        let mut event_handlers = self.event_handlers.write().await;
        event_handlers.push(OwnedEventHandler::new(event_handler))
    }

    pub async fn send<T: JdwpCommand>(&self, command: T) -> Result<T::Reply, io::Error> {
        let mut buffer = BytesMut::new();
        command.encode(&*self.codec.read().await, &mut buffer);
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let raw = RawCommandPacket::new_command(id, T::command_data(), buffer.freeze());
        let (tx, rx) = tokio::sync::oneshot::channel::<RawReplyPacket>();
        self.one_shots.write().await.insert(id, tx);
        self.raw_packet_sink.lock().await.send(raw).await?;

        let reply = rx.await.map_err(|e| Error::new(ErrorKind::BrokenPipe, e))?;

        let codec = self.codec.read().await;
        let mut decoder = JdwpDecoder::new(&*codec, reply.data().clone());

        let reply = decoder
            .get::<T::Reply>()
            .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))?;
        Ok(reply)
    }
}

/// creates a client
async fn create_client(
    mut input: OwnedReadHalf,
    mut output: OwnedWriteHalf,
) -> io::Result<JdwpClient> {
    handshake(&mut input, &mut output).await?;
    let raw_client = RawJdwpClient::new(input, output);
    let event_handlers = Arc::new(RwLock::new(Vec::<OwnedEventHandler<io::Error>>::new()));

    let mut join_set = JoinSet::<()>::new();
    let (event_tx, event_rx) = unbounded_channel::<Events>();
    {
        let mut event_handlers = event_handlers.clone();
        join_set.spawn(event_handling_loop(event_rx, event_handlers.clone()));
    }

    let (raw_sink, mut raw_stream) = raw_client.into_split();
    let codec = Arc::new(RwLock::new(JdwpCodec::default()));
    let one_shots = Arc::new(RwLock::new(
        HashMap::<u32, OneshotSender<RawReplyPacket>>::new(),
    ));

    {
        let codec = codec.clone();
        let one_shots = one_shots.clone();
        join_set.spawn(async move {
            let span = error_span!("packet-recv-loop");
            let _enter = span.enter();
            while let Some(raw_event) = raw_stream.next().await {
                let raw_event = raw_event.expect("getting packet failed");
                let codec = codec.read().await;
                match raw_event {
                    AnyRawPacket::Command(command) => {
                        trace!("got command {command:?} from JVM");

                        match to_events(command, &*codec) {
                            Ok(events) => {
                                event_tx.send(events).expect("event sender dropped");
                            }
                            Err(e) => {
                                warn!("Received unexpected command from JVM: {e}")
                            }
                        }
                    }
                    AnyRawPacket::Reply(reply) => {
                        trace!("got reply {reply:?} from JVM");
                        let id = reply.header().id();
                        if let Some(sender) = one_shots.write().await.remove(&id) {
                            sender.send(reply).expect("could not send");
                        }
                    }
                }
                trace!("waiting for next packet from JVM...");
            }
        });
    }

    let client = JdwpClient {
        id_sizes: Default::default(),
        tasks: join_set,
        event_handlers: event_handlers,
        raw_packet_sink: Mutex::from(raw_sink),
        next_id: AtomicU32::new(1),
        codec,
        one_shots,
    };
    Ok(client)
}

fn event_handling_loop(
    mut event_rx: UnboundedReceiver<Events>,
    mut event_handlers: Arc<RwLock<Vec<OwnedEventHandler<io::Error>>>>,
) -> impl Future<Output = ()> + Sized {
    async move {
        let mut buffered = VecDeque::<Events>::new();
        while let Some(events) = event_rx.recv().await {
            let event_handlers = event_handlers.read().await;
            if event_handlers.is_empty() {
                buffered.push_back(events);
            } else {
                let mut join_set = JoinSet::new();
                for buffered in buffered.drain(..) {
                    for event_handler in &*event_handlers {
                        for event in &buffered.events {
                            join_set.spawn(
                                event_handler
                                    .clone()
                                    .handle_event(buffered.policy, event.clone()),
                            );
                        }
                    }
                }
                for event_handler in &*event_handlers {
                    for event in &events.events {
                        join_set.spawn(
                            event_handler
                                .clone()
                                .handle_event(events.policy, event.clone()),
                        );
                    }
                }
                if let Err(e) = join_set
                    .join_all()
                    .await
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()
                {
                    error!("error handling events: {}", e);
                }
            }
        }
    }
}

#[instrument(skip_all, ok, err)]
async fn handshake<I, O>(mut input: I, output: &mut O) -> io::Result<()>
where
    I: AsyncRead + Unpin,
    O: AsyncWrite + Unpin,
{
    trace!("writing {JDWP_HANDSHAKE:?} to output stream");
    output.write_all(JDWP_HANDSHAKE).await?;
    let mut buf = [0u8; 14];
    trace!("waiting to read {JDWP_HANDSHAKE:?} from input stream");
    input.read_exact(&mut buf).await?;
    trace!("read {buf:?} from input stream");
    if &buf == JDWP_HANDSHAKE {
        trace!("Handshake matched");
        Ok(())
    } else {
        warn!("Handshake did not match");
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Expected JDWP handshake back in response",
        ))
    }
}
