mod transports;

use transports::Transport;
use transports::Transporter;
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("transport: {0}")]
    Transport(#[from] transports::Error),
}

pub struct Client {
    transport: Transport,
}

impl Client {
    pub async fn new(url: &Url) -> Result<Self, Error> {
        let transport = Transport::with_url(url).await?;
        Ok(Self::with_transport(transport))
    }

    pub fn with_transport(transport: Transport) -> Self {
        Self { transport }
    }
}
