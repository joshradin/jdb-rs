//! The traits and structs used to develop new [TransportService] implementations

use jdwp_client::JdwpClient;
use std::io;

/// Defines a transport service for connections between a debugger and a target VM.
pub trait TransportService: 'static {
    type Capabilities: TransportCapabilities;
    type Transport: jdwp_client::connect::JdwpTransport;

    /// Gets the capabilities of the transport service
    fn capabilities(&self) -> &Self::Capabilities;

    #[expect(async_fn_in_trait)]
    async fn connect(&self) -> io::Result<JdwpClient<Self::Transport>>;
}

/// The transport service capabilities
pub trait TransportCapabilities {
    /// Whether this transport service supports accept timeout
    fn accept_timeout(&self) -> bool;
    /// Whether this transport service supports attach timeout
    fn attach_timeout(&self) -> bool;
    /// Whether this transport service supports handshake timeout
    fn handshake_timeout(&self) -> bool;
    /// Whether this transport service can support multiple concurrent connections to
    /// a single address that it is listening on.
    fn multiple_connection(&self) -> bool;
}
