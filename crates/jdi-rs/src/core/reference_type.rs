use crate::{Mirror, VirtualMachine};

/// A type of an object in a target vm
pub trait ReferenceType<VM: VirtualMachine + ?Sized>: Mirror<VM> + PartialOrd {}
