//! futures

use jdwp_client::connect::JdwpTransport;
use std::future::Future;
use std::pin::Pin;

pub mod all_classes;
pub mod class;
pub mod reference_type;
type BoxedFuture<T> = Pin<Box<dyn Future<Output = T>>>;
