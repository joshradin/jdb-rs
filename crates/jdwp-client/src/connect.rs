//! defines how a client can connect to a target jvm

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

/// A type that can be used as a transport
pub trait JdwpTransport {
    type Input: AsyncRead + Unpin + Send + 'static;
    type Output: AsyncWrite + Unpin + Send + 'static;

    fn split_transport(self) -> (Self::Input, Self::Output)
    where
        Self: Sized;
}

impl JdwpTransport for TcpStream {
    type Input = OwnedReadHalf;
    type Output = OwnedWriteHalf;

    fn split_transport(self) -> (Self::Input, Self::Output)
    where
        Self: Sized,
    {
        self.into_split()
    }
}
