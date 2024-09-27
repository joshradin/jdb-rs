use jdwp_client::JdwpClient;
use regex::Regex;
use std::io;
use std::num::NonZeroU16;
use std::path::Path;
use std::process::Stdio;
use std::str::FromStr;
use std::sync::LazyLock;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};
use tokio::net::TcpStream;
use tokio::process::{Child, ChildStderr, ChildStdout, Command};

static DEBUG_STARTUP_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^Listening for transport dt_socket at address: (?<port>\d{1,5})$").unwrap()
});

/// A running java instance
#[derive(Debug)]
pub struct JavaInstance {
    port: NonZeroU16,
    child: Child,
    stdout: BufReader<ChildStdout>,
    stderr: BufReader<ChildStderr>,
}

impl JavaInstance {
    /// Starts a new running java instance, with debug enabled at a given port
    pub async fn new(debug_port: u16, main: impl AsRef<Path>) -> io::Result<Self> {
        let path = Path::new(env!("OUT_DIR")).join(main);
        let mut child = Command::new("java")
            .arg(format!(
                "-agentlib:jdwp=transport=dt_socket,server=y,address={debug_port},suspend=y"
            ))
            .arg(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdout = BufReader::new(child.stdout.take().expect("no stdout"));
        let mut stderr = BufReader::new(child.stderr.take().expect("no stderr"));

        let mut port: Option<NonZeroU16> = None;

        while port.is_none() {
            let mut buffer = String::new();
            stdout.read_line(&mut buffer).await?;
            if let Some(captures) = DEBUG_STARTUP_PATTERN.captures(buffer.trim()) {
                let port_raw = &captures["port"];
                let assigned_port = u16::from_str(port_raw).expect("cannot parse port");
                port = Some(NonZeroU16::new(assigned_port).unwrap());
            }
        }

        Ok(Self {
            port: port.unwrap(),
            child,
            stdout,
            stderr,
        })
    }

    pub fn port(&self) -> u16 {
        self.port.get()
    }
}

impl Drop for JavaInstance {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}
