pub mod client;
pub mod server;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum Error {
    #[error("io: {0}")]
    IO(String),
    #[error("json: {0}")]
    Json(String),
    #[error("jsonrpsee: {0}")]
    JsonRpsee(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}

impl From<jsonrpsee::types::Error> for Error {
    fn from(err: jsonrpsee::types::Error) -> Self {
        Self::JsonRpsee(err.to_string())
    }
}

use jsonrpsee::proc_macros::rpc;

#[cfg_attr(all(feature = "client", not(feature = "server")), rpc(client))]
#[cfg_attr(all(feature = "server", not(feature = "client")), rpc(server))]
#[cfg_attr(all(feature = "client", feature = "server"), rpc(client, server))]
#[cfg(any(feature = "server", feature = "client"))]
pub trait Rpc {
    #[method(name = "insert")]
    async fn insert(&self, params: client::insert::Request) -> Result<(), Error>;

    #[method(name = "open")]
    async fn open(&self, params: client::open::Request) -> Result<(), Error>;
}

#[cfg(feature = "server")]
pub use self::RpcServer as Server;

#[cfg(feature = "client")]
pub use self::RpcClient as Client;

#[cfg(feature = "server")]
pub async fn run_ws(
    server: impl Server,
    address: impl tokio::net::ToSocketAddrs,
) -> Result<(), Error> {
    use jsonrpsee::ws_server::WsServerBuilder;

    let ws_server = WsServerBuilder::default().build(address).await?;
    ws_server.start(server.into_rpc()).await;

    Ok(())
}

#[cfg(feature = "client")]
pub async fn connect_ws(url: &url::Url) -> Result<impl RpcClient, Error> {
    use jsonrpsee::types::traits::Client;
    use jsonrpsee::ws_client::WsClientBuilder;

    let client = WsClientBuilder::default().build(url.as_str()).await?;
    Ok(client)
}
