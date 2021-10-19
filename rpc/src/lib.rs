pub mod client;
pub mod server;
mod jsonrpc;

pub use jsonrpc::Id;
use serde::Deserialize;
use serde::Serialize;

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