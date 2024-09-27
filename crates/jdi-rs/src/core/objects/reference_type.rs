use crate::{Mirror, VirtualMachine};
use jdwp_types::{ClassStatus, ReferenceTypeId, TypeTag};
use std::sync::Weak;

#[derive(Debug)]
pub struct ReferenceType<VM: VirtualMachine + ?Sized> {
    type_tag: TypeTag,
    id: ReferenceTypeId,
    signature: String,
    status: ClassStatus,
    vm: Weak<VM>,
}

impl<VM: VirtualMachine + ?Sized> ReferenceType<VM> {
    /// Creates a new reference type
    pub fn new(
        type_tag: TypeTag,
        id: ReferenceTypeId,
        signature: String,
        status: ClassStatus,
        client: &Weak<VM>,
    ) -> Self {
        Self {
            type_tag,
            id,
            signature,
            status,
            vm: client.clone(),
        }
    }

    /// Gets the signature of the reference type
    pub fn signature(&self) -> &str {
        &self.signature
    }

    /// Gets the class status of the signature
    pub fn status(&self) -> ClassStatus {
        self.status
    }
}

impl<Vm: VirtualMachine + ?Sized> Mirror<Vm> for ReferenceType<Vm> {
    fn virtual_machine(&self) -> Weak<Vm> {
        self.vm.clone()
    }
}
