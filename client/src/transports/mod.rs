pub mod tcp;

use bytes::BytesMut;
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("URL Scheme is not supported")]
    UnsupportedURLScheme,
    #[error("IO: {0}")]
    IO(#[from] std::io::Error),
}

pub enum Transport {
    Tcp(tcp::Transport),
}

use async_trait::async_trait;

#[async_trait]
pub trait Transporter: Sized {
    async fn with_url(url: &Url) -> Result<Self, Error>;
    async fn read(&mut self, buf: &mut BytesMut) -> Result<usize, Error>;
    async fn write(&mut self, bytes: &[u8]) -> Result<(), Error>;
}

#[async_trait]
impl Transporter for Transport {
    async fn with_url(url: &Url) -> Result<Self, Error> {
        match url.scheme() {
            "tcp" => tcp::Transport::with_url(url).await.map(Transport::Tcp),
            _ => Err(Error::UnsupportedURLScheme),
        }
    }

    async fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        match self {
            Transport::Tcp(t) => t.write(bytes).await,
        }
    }

    async fn read(&mut self, buf: &mut BytesMut) -> Result<usize, Error> {
        match self {
            Transport::Tcp(t) => t.read(buf).await,
        }
    }
}
