pub mod client;
pub mod server;
mod jsonrpc;

use url::Url;
use serde::Serialize;
use serde::Deserialize;

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum Error {
    #[error("transport: {0}")]
    Transport(String),
    #[error("json: {0}")]
    Json(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Transport(err.to_string())
    }
}


impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}

#[cfg(feature = "client")]
pub async fn connect(client_peer: impl client::Peer, url: Url) -> Result<Box<dyn server::Peer>, Error>  {
    let server_peer = match url.scheme() {
        "ws" => Box::new(client::ws::Transport::new(client_peer, url).await?),
        _ => return Err(Error::Transport(format!("unrecognized scheme: {}", url.scheme())))

    };

    Ok(server_peer)
}

#[cfg(feature = "server")]
pub async fn run(server_peer: impl server::Peer, address: impl tokio::net::ToSocketAddrs) -> Result<(), Error>  {
    server::ws::run(server_peer, address).await?;
    Ok(())
}