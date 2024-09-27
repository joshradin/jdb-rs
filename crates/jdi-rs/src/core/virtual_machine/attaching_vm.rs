use crate::connect::spi::TransportService;
use crate::connect::Transport;
use crate::core::objects::all_classes::AllClasses;
use crate::core::private::VirtualMachineExt;
use crate::{Mirror, VirtualMachine};
use jdwp_client::connect::JdwpTransport;
use jdwp_client::JdwpClient;
use std::fmt::{Debug, Formatter, Pointer};
use std::sync::{Arc, Weak};

/// An attaching vm
pub struct AttachingVm<T: Transport>
where
    <T::TransportService as TransportService>::Transport: 'static,
{
    this: Weak<Self>,
    jdwp_client: Arc<JdwpClient<<T::TransportService as TransportService>::Transport>>,
}

impl<T: Transport> AttachingVm<T>
where
    <T::TransportService as TransportService>::Transport: 'static,
{
    /// Create a new attached VM
    pub fn new(
        jdwp_client: JdwpClient<<T::TransportService as TransportService>::Transport>,
    ) -> Arc<Self> {
        Arc::new_cyclic(|weak| Self {
            this: weak.clone(),
            jdwp_client: Arc::new(jdwp_client),
        })
    }
}

impl<T: Transport> VirtualMachineExt for AttachingVm<T>
where
    <T::TransportService as TransportService>::Transport: 'static,
{
    fn client(&self) -> &JdwpClient<impl jdwp_client::connect::JdwpTransport> {
        &self.jdwp_client
    }
}

impl<T: Transport> Debug for AttachingVm<T>
where
    <T::TransportService as TransportService>::Transport: 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttachingVm")
            .field("jdwp_client", &self.jdwp_client)
            .finish()
    }
}

impl<T: Transport> Mirror<Self> for AttachingVm<T>
where
    <T::TransportService as TransportService>::Transport: 'static,
{
    fn virtual_machine(&self) -> Weak<Self> {
        self.this.clone()
    }
}

impl<T: Transport> VirtualMachine for AttachingVm<T>
where
    <T::TransportService as TransportService>::Transport: 'static,
{
    type Transport = T;

    fn all_classes(&self) -> AllClasses<Self> {
        AllClasses::new(&self.this)
    }
}
