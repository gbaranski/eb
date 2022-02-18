use std::{collections::HashMap, net::SocketAddr};

use eb_core::SessionID;
use futures::stream::{SplitSink, StreamExt};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_tungstenite::{tungstenite, WebSocketStream};
use url::Url;

type ServerName = &'static str;

#[derive(Debug)]
pub enum ServerMessage {
    Connected(SessionID, SessionHandle, SocketAddr),
    Disconnected(SessionID),
    Insert { cursor: usize, content: String },
    Open { url: Url },
}

impl acu::Message for ServerMessage {}

struct Server {
    receiver: acu::Receiver<ServerMessage, ServerName>,
    sessions: HashMap<SessionID, SessionHandle>,
    content: String,
    file: Option<Url>,
}

impl Server {
    async fn listen(
        address: impl ToSocketAddrs,
        handle: ServerHandle,
    ) -> Result<(), anyhow::Error> {
        let listener = TcpListener::bind(address).await?;
        loop {
            let (stream, address) = listener.accept().await?;
            let stream = tokio_tungstenite::accept_async(stream).await.unwrap();
            let session_id = SessionID::new_v4();
            let (tx, mut rx) = stream.split();
            let session_handle = new_session(session_id, tx, handle.clone());
            handle.connected(session_id, session_handle.clone(), address).await;
            tokio::spawn(async move {
                while let Some(message) = rx.next().await {
                    let message = message?;
                    session_handle.websocket_message(message).await;
                    // let message = message?;
                    // handle.message(session_id, message);
                }
                Ok::<_, anyhow::Error>(())
            });
        }
    }

    async fn updated(&mut self) {
        tracing::info!("current content: {}", self.content);
    }

    async fn run(
        &mut self,
        address: impl ToSocketAddrs + Send + 'static,
        handle: ServerHandle,
    ) -> Result<(), anyhow::Error> {
        tokio::spawn(async move { Self::listen(address, handle).await.unwrap() });
        while let Some(message) = self.receiver.recv().await {
            match message {
                ServerMessage::Connected(session_id, session_handle, address) => {
                    tracing::info!("{} connected from {}", session_id, address);
                    self.sessions.insert(session_id, session_handle);
                }
                ServerMessage::Disconnected(session_id) => {
                    self.sessions.remove(&session_id);
                }
                ServerMessage::Insert { cursor, content } => {
                    self.content.insert_str(cursor, &content);
                    tracing::info!("Inserted {} at {}", content, cursor);
                    self.updated().await;
                }
                ServerMessage::Open { url } => {
                    self.file = Some(url);
                }
            };
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ServerHandle {
    pub sender: acu::Sender<ServerMessage, &'static str>,
}

impl ServerHandle {
    pub async fn connected(&self, session_id: SessionID, session_handle: SessionHandle, address: SocketAddr) {
        self.sender
            .send(ServerMessage::Connected(session_id, session_handle, address))
            .await
    }

    pub async fn disconnected(&self, session_id: SessionID) {
        self.sender
            .send(ServerMessage::Disconnected(session_id))
            .await
    }

    pub async fn insert(&self, cursor: usize, content: String) {
        self.sender
            .send(ServerMessage::Insert { cursor, content })
            .await
    }

    pub async fn open(&self, url: Url) {
        self.sender.send(ServerMessage::Open { url }).await
    }
}

pub fn new_server(address: impl ToSocketAddrs + Send + 'static) -> ServerHandle {
    let (sender, receiver) = acu::channel(8, "Server");
    let mut actor = Server {
        receiver,
        sessions: HashMap::new(),
        content: String::from("Placeholder!"),
        file: None,
    };
    let handle = ServerHandle { sender };
    tokio::spawn({
        let handle = handle.clone();
        async move { actor.run(address, handle.clone()).await }
    });
    handle
}

#[derive(Debug)]
pub enum SessionMessage {
    WebSocketMessage(tungstenite::Message),
}

impl acu::Message for SessionMessage {}

struct Session {
    id: SessionID,
    receiver: acu::Receiver<SessionMessage, SessionID>,
    sink: SplitSink<WebSocketStream<TcpStream>, tungstenite::Message>,
    server: ServerHandle,
}

impl Session {
    async fn run(&mut self) -> Result<(), anyhow::Error> {
        while let Some(message) = self.receiver.recv().await {
            match message {
                SessionMessage::WebSocketMessage(message) => match message {
                    tungstenite::Message::Text(text) => {
                        use eb_core::client;
                        let json: client::Frame = serde_json::from_str(&text).unwrap();
                        match json {
                            client::Frame::Insert(client::Insert { cursor, content }) => {
                                self.server.insert(cursor, content).await
                            }
                            client::Frame::Open(client::Open { url }) => {
                                self.server.open(url).await
                            }
                            _ => unimplemented!(),
                        }
                    }
                    tungstenite::Message::Binary(_) => todo!(),
                    tungstenite::Message::Ping(_) => todo!(),
                    tungstenite::Message::Pong(_) => todo!(),
                    tungstenite::Message::Close(_) => self.server.disconnected(self.id).await,
                    tungstenite::Message::Frame(_) => todo!(),
                },
            };
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SessionHandle {
    pub sender: acu::Sender<SessionMessage, SessionID>,
}

impl SessionHandle {
    pub async fn websocket_message(&self, message: tungstenite::Message) {
        self.sender
            .send(SessionMessage::WebSocketMessage(message))
            .await
    }
}

pub fn new_session(
    id: SessionID,
    sink: SplitSink<WebSocketStream<TcpStream>, tungstenite::Message>,
    server: ServerHandle,
) -> SessionHandle {
    let (sender, receiver) = acu::channel(8, id);
    let mut actor = Session {
        id,
        receiver,
        sink,
        server,
    };
    let handle = SessionHandle { sender };
    tokio::spawn(async move { actor.run().await });
    handle
}
