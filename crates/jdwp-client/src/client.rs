use crate::events::EventHandler;
use crate::events::{Event, OwnedEventHandler};
use crate::id_sizes::IdSizes;
use crate::raw::codec::RawCodec;
use crate::raw::packet::{AnyRawPacket, RawCommandPacket};
use crate::raw::RawJdwpClient;
use futures_util::task::SpawnExt;
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use std::collections::VecDeque;
use std::future::Future;
use std::io;
use std::io::Error;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio::sync::{Mutex, RwLock};
use tokio::task::{JoinHandle, JoinSet};
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{error, error_span, instrument, trace, warn, Instrument};

static JDWP_HANDSHAKE: &[u8; 14] = b"JDWP-Handshake";

/// A non-blocking jdwp client
pub struct JdwpClient {
    id_sizes: IdSizes,
    tasks: JoinSet<()>,
    event_handlers: Arc<RwLock<Vec<OwnedEventHandler<io::Error>>>>,
}

impl JdwpClient {
    /// Creates a new jdwp client over a tcp stream
    pub async fn create(stream: TcpStream) -> io::Result<Self> {
        let (input, output) = stream.into_split();
        create_client(input, output).await
    }

    pub async fn on_event<E : EventHandler<Err=io::Error> + Sync>(&mut self, event_handler: E) {
        let mut event_handlers = self.event_handlers.write().await;
        event_handlers.push(OwnedEventHandler::new(event_handler))
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
    let (event_tx, event_rx) = unbounded_channel::<Event>();
    {
        let mut event_handlers = event_handlers.clone();
        join_set.spawn(event_handling_loop(event_rx, event_handlers.clone()));
    }

    let (raw_sink, mut raw_stream) = raw_client.into_split();

    join_set.spawn(async move {
        let span = error_span!("packet-recv-loop");
        let _enter = span.enter();
        while let Some(raw_event) =raw_stream.next().await{
            match raw_event {
                AnyRawPacket::Command(command) => {
                    trace!("got command {command:?} from JVM");
                }
                AnyRawPacket::Reply(_) => {}
            }
        }
    });

    Ok(JdwpClient {
        id_sizes: Default::default(),
        tasks: join_set,
        event_handlers: event_handlers,
    })
}

fn event_handling_loop(
    mut event_rx: UnboundedReceiver<Event>,
    mut event_handlers: Arc<RwLock<Vec<OwnedEventHandler<io::Error>>>>,
) -> impl Future<Output = ()> + Sized {
    async move {
        let mut buffered = VecDeque::<Event>::new();
        while let Some(event) = event_rx.recv().await {
            let event_handlers = event_handlers.read().await;
            if event_handlers.is_empty() {
                buffered.push_back(event);
            } else {
                let mut join_set = JoinSet::new();
                for buffered in buffered.drain(..) {
                    for event_handler in &*event_handlers {
                        join_set.spawn(event_handler.clone().handle_event(buffered.clone()));
                    }
                }
                for event_handler in &*event_handlers {
                    join_set.spawn(event_handler.clone().handle_event(event.clone()));
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
