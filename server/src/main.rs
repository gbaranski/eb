use bytes::BytesMut;
use eb_core::client::Message as ClientMessage;
use eb_core::server::Message as ServerMessage;
use ropey::Rope;
use std::net::Ipv4Addr;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

const LOG_ENV: &str = "EB_SERVER_LOG";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    use std::str::FromStr;
    use tracing_subscriber::EnvFilter;

    let env_filter = match std::env::var(LOG_ENV) {
        Ok(env) => env,
        Err(std::env::VarError::NotPresent) => "info".to_string(),
        Err(std::env::VarError::NotUnicode(_)) => panic!(
            "{} environment variable is not valid unicode and can't be read",
            LOG_ENV
        ),
    };
    let env_filter = EnvFilter::from_str(&env_filter)
        .unwrap_or_else(|err| panic!("invalid `{}` environment variable {}", LOG_ENV, err));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
    let server = Server::new().await?;
    server.run().await?;
    Ok(())
}

struct Server {
    tcp_listener: TcpListener,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("io: {0}")]
    IO(#[from] std::io::Error),
    #[error("json: {0}")]
    JSON(#[from] serde_json::Error),
}

impl Server {
    pub async fn new() -> Result<Self, Error> {
        let tcp_listener =
            TcpListener::bind((Ipv4Addr::LOCALHOST, eb_core::DEFAULT_TCP_PORT)).await?;
        Ok(Self { tcp_listener })
    }

    pub async fn run(self) -> Result<(), Error> {
        tracing::info!("Starting server");
        loop {
            let (stream, address) = self.tcp_listener.accept().await?;
            let session = Session {
                stream,
                rope: Rope::default(),
                cursor: 0,
            };
            tokio::spawn(async move {
                tracing::info!(address = %address, "Session started");
                match session.run().await {
                    Ok(_) => {
                        tracing::info!(address = %address, "Session closed");
                    }
                    Err(err) => {
                        tracing::error!(address = %address, "Session error: {}", err);
                    }
                }
            });
        }
    }
}

struct Session {
    stream: TcpStream,
    rope: Rope,
    cursor: usize,
}

impl Session {
    pub async fn run(mut self) -> Result<(), Error> {
        let mut buf = BytesMut::with_capacity(4096); // What if message exceeds 4096 bytes?
        loop {
            buf.clear();
            let n = self.stream.read_buf(&mut buf).await?;
            tracing::debug!("read message. n = {}", n);
            if n == 0 {
                return Ok(());
            }

            tracing::debug!("raw message = {}", std::str::from_utf8(&buf[0..n]).unwrap());
            let message: ClientMessage = serde_json::from_slice(&buf[0..n])?;
            tracing::debug!("message = {:?}", message);
            match message {
                ClientMessage::Insert { content } => {
                    self.rope.insert(self.cursor, &content);
                    self.send(&ServerMessage::Insert { content }).await?;
                }
            };
        }
    }

    async fn send(&mut self, message: &ServerMessage) -> Result<(), Error> {
        let bytes = serde_json::to_vec(message)?;
        self.stream.write(&bytes).await?;
        Ok(())
    }
}
