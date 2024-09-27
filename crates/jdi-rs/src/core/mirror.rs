use crate::VirtualMachine;
use std::fmt::Debug;
use std::sync::Weak;

/// A proxy used by the debugger to examine or manipulate an entity in another virtual machine
pub trait Mirror<VM: VirtualMachine + ?Sized>: Debug {
    /// Gets the virtual machine this mirror belongs to.
    fn virtual_machine(&self) -> Weak<VM>;
}
