use super::Error;
use super::Transporter;
use async_trait::async_trait;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use url::Url;

pub struct Transport {
    stream: TcpStream,
}

#[async_trait]
impl Transporter for Transport {
    async fn with_url(url: &Url) -> Result<Self, Error> {
        assert_eq!(url.scheme(), "tcp");
        let stream = TcpStream::connect((
            url.host().unwrap().to_string(),
            url.port().unwrap_or(eb_core::DEFAULT_TCP_PORT),
        ))
        .await?;

        Ok(Self { stream })
    }

    async fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.stream.write(bytes).await?;
        Ok(())
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let n = self.stream.read(buf).await?;
        Ok(n)
    }
}
