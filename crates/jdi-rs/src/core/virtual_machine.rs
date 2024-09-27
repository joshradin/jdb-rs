use crate::connect::Transport;
use crate::core::objects::all_classes::AllClasses;
use crate::core::private::VirtualMachineExt;
use crate::Mirror;
use jdwp_client::connect::JdwpTransport;

pub mod attaching_vm;

/// A virtual machine
pub trait VirtualMachine: VirtualMachineExt + Mirror<Self> + 'static {
    type Transport: Transport;

    fn all_classes(&self) -> AllClasses<Self>;
}
