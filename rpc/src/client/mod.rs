pub mod ws;

use crate::jsonrpc;
use crate::Error;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::borrow::Cow;
use url::Url;

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

use async_trait::async_trait;

#[async_trait]
pub trait Transport {
    async fn send(&self, message: jsonrpc::Message) -> Result<(), Error>;
    async fn recv(&mut self) -> Result<jsonrpc::Message, Error>;
}

pub struct Client {
    transport: Box<dyn Transport>,
}

impl Client {
    pub async fn connect(url: Url) -> Result<Self, Error> {
        let transport = match url.scheme() {
            "ws" => Box::new(ws::Transport::new(url).await?),
            _ => {
                return Err(Error::Transport(format!(
                    "unrecognized scheme: {}",
                    url.scheme()
                )))
            }
        };

        Ok(Self {
            transport
        })
    }

    pub async fn call<T: DeserializeOwned>(
        &mut self,
        params: impl Serialize,
        method: &'static str,
        id: jsonrpc::Id,
    ) -> Result<T, Error> {
        let message = jsonrpc::Message::Request(jsonrpc::Request {
            jsonrpc: jsonrpc::Version,
            method: Cow::from(method),
            kind: jsonrpc::RequestKind::Request {
                params: serde_json::to_value(params).unwrap(),
                id: id.clone(),
            },
        });
        self.transport.send(message).await?;
        loop {
            let message = self.transport.recv().await?;
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
