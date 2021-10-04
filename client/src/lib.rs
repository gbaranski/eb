mod transports;

use bytes::BytesMut;
use eb_core::client::Message as ClientMessage;
use eb_core::server::Message as ServerMessage;
use transports::Transport;
use transports::Transporter;
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("transport: {0}")]
    Transport(#[from] transports::Error),
    #[error("json: {0}")]
    JSON(#[from] serde_json::Error),
}

pub struct Client {
    transport: Transport,
    buf: BytesMut,
}

impl Client {
    pub async fn new(url: &Url) -> Result<Self, Error> {
        let transport = Transport::with_url(url).await?;
        Ok(Self::with_transport(transport))
    }

    pub fn with_transport(transport: Transport) -> Self {
        Self {
            transport,
            buf: BytesMut::with_capacity(4096),
        }
    }

    pub async fn send(&mut self, message: &ClientMessage) -> Result<(), Error> {
        let bytes = serde_json::to_vec(message)?;
        self.transport.write(&bytes).await?;
        tracing::info!(message = ?message, bytes = ?bytes, "sent");
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Option<ServerMessage>, Error> {
        self.buf.clear();
        let n = self.transport.read(&mut self.buf).await?;
        if n == 0 {
            return Ok(None);
        } else {
            tracing::debug!(
                "Received message: {:?}",
                std::str::from_utf8(&self.buf[0..n])
            );
            let message = serde_json::from_slice(&self.buf[0..n])?;
            Ok(message)
        }
    }
}
