use crate::jsonrpc;
use crate::Error;
use async_trait::async_trait;
use futures::Sink;
use futures::SinkExt;
use futures::Stream;
use futures::StreamExt;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite;
use url::Url;

use super::Peer;

impl From<tungstenite::Error> for Error {
    fn from(err: tungstenite::Error) -> Self {
        Self::Transport(err.to_string())
    }
}

pub struct Transport {
    request_tx: mpsc::UnboundedSender<jsonrpc::Message>,
    response_rx: broadcast::Receiver<jsonrpc::Message>,
}

impl Transport {
    pub async fn new(peer: impl Peer, url: Url) -> Result<Self, Error> {
        let (stream, _) = tokio_tungstenite::connect_async(url).await?;
        let (stream_tx, stream_rx) = stream.split();
        let (request_tx, request_rx) = mpsc::unbounded_channel();
        let (response_tx, response_rx) = broadcast::channel(16);

        tokio::spawn(async move {
            Self::stream_write(stream_tx, request_rx).await.unwrap();
        });

        tokio::spawn(async move {
            Self::stream_read(stream_rx, response_tx).await.unwrap();
        });

        Ok(Self {
            request_tx,
            response_rx,
        })
    }

    async fn stream_write(
        mut stream_tx: impl Sink<tungstenite::Message, Error = tungstenite::Error> + Unpin,
        mut request_rx: mpsc::UnboundedReceiver<jsonrpc::Message>,
    ) -> Result<(), Error> {
        while let Some(message) = request_rx.recv().await {
            let s = match message {
                jsonrpc::Message::Request(request) => serde_json::to_string(&request),
                jsonrpc::Message::Response(response) => serde_json::to_string(&response),
            }?;
            stream_tx.send(tungstenite::Message::Text(s)).await?;
        }
        Ok(())
    }

    async fn stream_read(
        mut stream_rx: impl Stream<Item = Result<tungstenite::Message, tungstenite::Error>> + Unpin,
        response_tx: broadcast::Sender<jsonrpc::Message>,
    ) -> Result<(), Error> {
        while let Some(message) = stream_rx.next().await {
            let message = message?;
            match message {
                tungstenite::Message::Text(text) => {
                    let message = serde_json::from_str(&text)?;
                    response_tx
                        .send(message)
                        .map_err(|_| Error::Transport(String::from("response_tx is closed")))?;
                }
                _ => unimplemented!(),
            };
        }
        Ok(())
    }
}

#[async_trait]
impl super::Transport for Transport {
    async fn send(&self, message: jsonrpc::Message) -> Result<(), Error> {
        self.request_tx
            .send(message)
            .map_err(|_| Error::Transport(String::from("request_tx is closed")))?;
        Ok(())
    }

    async fn recv(&mut self) -> Result<jsonrpc::Message, Error> {
        self.response_rx
            .recv()
            .await
            .map_err(|_| Error::Transport(String::from("request_tx is closed")))?;
        todo!()
    }

    fn get_id(&self) -> usize {
        todo!()
    }
}
