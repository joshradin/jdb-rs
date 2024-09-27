//! Private stuff

use crate::VirtualMachine;
use jdwp_client::connect::JdwpTransport;
use jdwp_client::JdwpClient;

/// An extension of a virtual machine, only accessible within this crate
pub trait VirtualMachineExt {
    /// Get access to the underlying transport used by this
    fn client(&self) -> &JdwpClient<impl JdwpTransport>;
}
