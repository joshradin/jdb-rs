//! main entry point

use crate::connect::spi::TransportService;
use crate::connect::{Connector, TcpAttachingConnector, Transport};
use crate::core::virtual_machine::attaching_vm::AttachingVm;
use crate::VirtualMachine;
use std::io;
use std::sync::Arc;
use tokio::net::ToSocketAddrs;

/// A manager of connections to target virtual machines
#[derive(Debug)]
pub struct VirtualMachineManager;

impl VirtualMachineManager {
    /// Attach to a previously running socket address
    pub async fn attach<A: ToSocketAddrs>(addr: A) -> io::Result<Arc<impl VirtualMachine>> {
        let result = TcpAttachingConnector::tcp(addr).await?;
        let client = result.transport().service().connect().await?;
        let attaching = AttachingVm::<<TcpAttachingConnector as Connector>::Transport>::new(client);
        Ok(attaching)
    }
}
