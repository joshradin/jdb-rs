use jdb_test_fixtures::JavaInstance;
use jdwp_client::JdwpClient;
use std::io;
use tokio::net::TcpStream;

pub trait JdwpJavaInstanceExt {
    async fn connect(&self) -> io::Result<JdwpClient<TcpStream>>;
}

impl JdwpJavaInstanceExt for JavaInstance {
    async fn connect(&self) -> io::Result<JdwpClient<TcpStream>> {
        let tcp_stream = TcpStream::connect(("127.0.0.1", self.port())).await?;
        JdwpClient::create(tcp_stream).await
    }
}
