#[cfg(feature = "ws-client")]
mod ws;

use crate::jsonrpc;
use serde::Deserialize;
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Update(update::Request),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Update(update::Response),
}

pub mod update {
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Request {
        pub content: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Response {}
}

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum Error {
    #[error("transport: {0}")]
    Transport(String),
    #[error("json: {0}")]
    Json(String),
}

use async_trait::async_trait;

#[async_trait]
pub trait Client {
    async fn update(&mut self, request: update::Request) -> Result<update::Response, Error>;
}

#[async_trait]
pub trait Transport {
    async fn send(&self, message: jsonrpc::Message) -> Result<(), Error>;
    async fn recv(&mut self) -> Result<jsonrpc::Message, Error>;
    fn get_id(&self) -> usize;
}

#[async_trait]
impl<C: Transport + Send + Sync> Client for C {
    async fn update(&mut self, request: update::Request) -> Result<update::Response, Error> {
        let id = jsonrpc::Id::Number(self.get_id() as i64);
        self.send(jsonrpc::Message::Request(jsonrpc::Request {
            jsonrpc: jsonrpc::Version,
            method: Cow::from("update"),
            kind: jsonrpc::RequestKind::Request {
                params: serde_json::to_value(request)?,
                id: id.clone(),
            },
        }))
        .await?;
        loop {
            let message = self.recv().await?;
            if let jsonrpc::Message::Response(response) = message {
                if response.id == id {
                    match response.kind {
                        jsonrpc::ResponseKind::Ok { result } => {
                            return Ok(serde_json::from_value(result)?);
                        }
                        jsonrpc::ResponseKind::Err { error } => {
                            panic!("error: {:?}", error);
                        }
                    };
                }
            }
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}
