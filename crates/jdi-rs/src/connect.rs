//! Defines connections between a client and a target virtual machine

use crate::connect::spi::TransportService;

pub mod spi;

/// A method of connection between a debugger and a target VM, encapsulating exactly
/// one [Transport].
pub trait Connector {
    type Transport: Transport;

    /// A short identifier for this transport
    fn name(&self) -> &str;
    /// Gets the transport for this connector
    fn transport(&self) -> &Self::Transport;
}

pub trait Transport: 'static {
    type TransportService: TransportService;

    /// A short identifier for this transport
    fn name(&self) -> &str;

    /// Gets the encapsulated transport service
    fn service(&self) -> &Self::TransportService;
}

mod connectors;
pub(crate) use connectors::*;
