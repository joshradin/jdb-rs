use crate::core::objects::reference_type::ReferenceType;
use crate::core::objects::BoxedFuture;
use crate::VirtualMachine;
use jdwp_client::commands;
use jdwp_client::commands::{AllClassesReply, ClassReferenceWithSignature};
use jdwp_client::connect::JdwpTransport;
use pin_project::pin_project;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Weak;
use std::task::{Context, Poll};

/// All Classes
#[pin_project]
pub struct AllClasses<VM: VirtualMachine + ?Sized> {
    vm: Weak<VM>,
    #[pin]
    future: Option<BoxedFuture<io::Result<AllClassesReply>>>,
}

impl<VM: VirtualMachine + ?Sized> Future for AllClasses<VM> {
    type Output = io::Result<Vec<ReferenceType<VM>>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut me = self.project();
        loop {
            if let Some(future) = me.future.as_mut().as_pin_mut().take() {
                match future.poll(cx) {
                    Poll::Ready(ready) => {
                        me.future.take();
                        match ready {
                            Ok(all_classes) => {
                                let mut vector = Vec::with_capacity(all_classes.classes.len());
                                for class in all_classes.classes {
                                    let ClassReferenceWithSignature {
                                        type_tag,
                                        id,
                                        signature,
                                        status,
                                    } = class;
                                    vector.push(ReferenceType::new(
                                        type_tag,
                                        id,
                                        signature,
                                        status,
                                        &me.vm,
                                    ));
                                }
                                return Poll::Ready(Ok(vector));
                            }
                            Err(err) => {
                                return Poll::Ready(Err(err));
                            }
                        }
                    }
                    Poll::Pending => {
                        return Poll::Pending;
                    }
                }
            } else {
                let vm = me.vm.upgrade().expect("vm is dead");
                let future = Box::pin(async move { vm.client().send(commands::AllClasses).await });
                let _ = me.future.insert(future);
            }
        }
    }
}

impl<'a, T: VirtualMachine> AllClasses<T> {
    pub fn new(client: &'a Weak<T>) -> Self {
        Self {
            vm: client.clone(),
            future: None,
        }
    }
}
