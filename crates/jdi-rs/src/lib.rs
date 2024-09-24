//! # `jdi-rs`
//!
//! Provides a 'jdi'-like interface over the [jdwp] protocol, as implemented by [jdwp-client](jdwp_client).
//!
//! Original description, as provided by Oracle[^1]:
//! > The Java Debug Interface is a high level rust api providing information for debuggers and similar
//! > systems needing access to the running state of a (usually) remote virtual machine.
//!
//!
//! [jdwp]: <https://docs.oracle.com/javase/8/docs/technotes/guides/jpda/jdwp-spec.html>
//!
//! [^1]: <https://docs.oracle.com/javase/8/docs/jdk/api/jpda/jdi/>

pub use core::*;

mod core;
pub mod connect;
pub mod event;
pub mod request;
