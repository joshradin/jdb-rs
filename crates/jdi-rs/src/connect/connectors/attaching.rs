use crate::connect::spi::{TransportCapabilities, TransportService};
use crate::connect::{Connector, Transport};
use futures::TryFutureExt;
use jdwp_client::JdwpClient;
use std::io;
use std::net::SocketAddr;
use tokio::net::{lookup_host, TcpStream, ToSocketAddrs};
use tracing::trace;

pub type TcpAttachingConnector = AttachingConnector<AttachingTcpStreamTransportService>;

/// A connector which attaches to a previously running target VM
#[derive(Debug)]
pub struct AttachingConnector<T: TransportService> {
    transport: AttachingTransport<T>,
}

impl TcpAttachingConnector {
    /// Creates a new [AttachingConnector], by trying to query a given socket addr
    pub async fn tcp<A: ToSocketAddrs>(addr: A) -> io::Result<TcpAttachingConnector> {
        let addrs = lookup_host(addr).await?;
        let cx = TcpAttachingConnector {
            transport: AttachingTransport {
                service: AttachingTcpStreamTransportService {
                    addresses: addrs.collect(),
                },
            },
        };
        Ok(cx)
    }
}

impl<T: TransportService> Connector for AttachingConnector<T> {
    type Transport = AttachingTransport<T>;

    fn name(&self) -> &str {
        "tcp-attach-connector"
    }

    fn transport(&self) -> &Self::Transport {
        &self.transport
    }
}

#[derive(Debug)]
pub struct AttachingTransport<T: TransportService> {
    service: T,
}

impl<T: TransportService> Transport for AttachingTransport<T> {
    type TransportService = T;

    fn name(&self) -> &str {
        "tcp-attach-transport"
    }

    fn service(&self) -> &Self::TransportService {
        &self.service
    }
}

#[derive(Debug)]
pub struct AttachingTcpStreamTransportService {
    addresses: Vec<SocketAddr>,
}

impl TransportService for AttachingTcpStreamTransportService {
    type Capabilities = AttachingTcpStreamTransportCapabilities;
    type Transport = TcpStream;

    fn capabilities(&self) -> &Self::Capabilities {
        &AttachingTcpStreamTransportCapabilities
    }

    async fn connect(&self) -> io::Result<JdwpClient<Self::Transport>> {
        for addr in &self.addresses {
            trace!("trying to connect to JDWP client at {addr:?}");
            if let Ok(client) = TcpStream::connect(addr)
                .and_then(|stream| JdwpClient::create(stream))
                .await
            {
                return Ok(client);
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "No client found",
        ))
    }
}

pub struct AttachingTcpStreamTransportCapabilities;

impl TransportCapabilities for AttachingTcpStreamTransportCapabilities {
    fn accept_timeout(&self) -> bool {
        false
    }

    fn attach_timeout(&self) -> bool {
        false
    }

    fn handshake_timeout(&self) -> bool {
        false
    }

    fn multiple_connection(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::connect::connectors::attaching::TcpAttachingConnector;
    use crate::connect::spi::TransportService;
    use crate::connect::{Connector, Transport};

    #[tokio::test]
    async fn test_create_attaching_service() {
        let create_service = TcpAttachingConnector::tcp("localhost:5005")
            .await
            .expect("could not create service");
    }
}
