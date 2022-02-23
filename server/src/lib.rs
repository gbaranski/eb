use chronofold::LogIndex;
use eb_core::client;
use eb_core::server;
use eb_core::Author;
use eb_core::Chronofold;
use eb_core::ClientID;
use futures::Sink;
use futures::{future, stream::StreamExt, SinkExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite;
use url::Url;

type ServerName = &'static str;

#[derive(Debug)]
pub enum ServerMessage {
    Connected(ClientID, SessionHandle, SocketAddr),
    Disconnected(ClientID),
    Insert(ClientID, LogIndex, char, oneshot::Sender<()>),
    Open { url: Url },
    Get { respond_to: oneshot::Sender<String> },
}

impl acu::Message for ServerMessage {}

struct Server {
    receiver: acu::Receiver<ServerMessage, ServerName>,
    sessions: HashMap<ClientID, SessionHandle>,
    content: Chronofold,
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
            let client_id = ClientID::new_v4();
            let (tx, mut rx) = stream.split();
            let session_handle = new_session(client_id, tx, handle.clone());
            handle
                .connected(client_id, session_handle.clone(), address)
                .await;
            tokio::spawn(async move {
                while let Some(message) = rx.next().await {
                    let message = message?;
                    match message {
                        tungstenite::Message::Text(text) => {
                            tracing::debug!("websocket message: {}", text);
                            let frame: client::Frame = serde_json::from_str(&text).unwrap();
                            session_handle.client_frame(frame).await;
                        }
                        tungstenite::Message::Binary(_) => todo!(),
                        tungstenite::Message::Ping(_) => todo!(),
                        tungstenite::Message::Pong(_) => todo!(),
                        tungstenite::Message::Close(_) => session_handle.close().await,
                        tungstenite::Message::Frame(_) => todo!(),
                    };
                }
                Ok::<_, anyhow::Error>(())
            });
        }
    }

    async fn update_sessions(&mut self) {
        let s = self.content.to_string();
        tracing::info!("current content: {}", s);
        let futures = self
            .sessions
            .values()
            .map(|handle| handle.update(s.clone()));
        future::join_all(futures).await;
    }

    async fn handle_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::Connected(client_id, session_handle, address) => {
                tracing::info!("{} connected from {}", client_id, address);
                session_handle.update(self.content.to_string()).await;
                self.sessions.insert(client_id, session_handle);
            }
            ServerMessage::Disconnected(client_id) => {
                self.sessions.remove(&client_id);
            }
            ServerMessage::Insert(client_id, index, char, respond_to) => {
                self.content
                    .session(Author::Client(client_id))
                    .insert_after(index, char);
                tracing::info!("insert `{}`", char);
                self.update_sessions().await;
                respond_to.send(()).unwrap();
            }
            ServerMessage::Open { url } => {
                self.file = Some(url);
            }
            ServerMessage::Get { respond_to } => respond_to.send(self.content.to_string()).unwrap(),
        };
    }

    async fn run(
        &mut self,
        address: impl ToSocketAddrs + Send + 'static,
        handle: ServerHandle,
    ) -> Result<(), anyhow::Error> {
        tokio::spawn(async move { Self::listen(address, handle).await.unwrap() });
        while let Some(message) = self.receiver.recv().await {
            self.handle_message(message).await;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ServerHandle {
    pub sender: acu::Sender<ServerMessage, &'static str>,
}

impl ServerHandle {
    pub async fn connected(
        &self,
        client_id: ClientID,
        session_handle: SessionHandle,
        address: SocketAddr,
    ) {
        self.sender
            .notify(ServerMessage::Connected(client_id, session_handle, address))
            .await
    }

    pub async fn disconnected(&self, client_id: ClientID) {
        self.sender
            .notify(ServerMessage::Disconnected(client_id))
            .await
    }

    pub async fn insert(&self, client_id: ClientID, index: LogIndex, char: char) {
        self.sender
            .call_with(|respond_to| ServerMessage::Insert(client_id, index, char, respond_to))
            .await
    }

    pub async fn open(&self, url: Url) {
        self.sender.notify(ServerMessage::Open { url }).await
    }

    pub async fn get(&self) -> String {
        self.sender
            .call_with(|respond_to| ServerMessage::Get { respond_to })
            .await
    }
}

pub fn new_server(address: impl ToSocketAddrs + Send + 'static) -> ServerHandle {
    let (sender, receiver) = acu::channel(8, "Server");
    let content = Chronofold::new(Author::Server);
    let mut actor = Server {
        receiver,
        sessions: HashMap::new(),
        content,
        file: None,
    };
    let handle = ServerHandle { sender };
    tokio::spawn({
        let handle = handle.clone();
        async move { actor.run(address, handle.clone()).await.unwrap() }
    });
    handle
}

#[derive(Debug)]
pub enum SessionMessage {
    ClientFrame {
        frame: client::Frame,
        respond_to: oneshot::Sender<()>,
    },
    Update(String),
    Close,
}

impl acu::Message for SessionMessage {}

struct Session<S: Sink<tungstenite::Message>> {
    id: ClientID,
    receiver: acu::Receiver<SessionMessage, ClientID>,
    sink: S,
    server: ServerHandle,
}

impl<
        SE: std::error::Error + Send + Sync + 'static,
        S: Sink<tungstenite::Message, Error = SE> + Unpin,
    > Session<S>
{
    async fn run(&mut self) -> Result<(), anyhow::Error> {
        while let Some(message) = self.receiver.recv().await {
            match message {
                SessionMessage::Update(content) => {
                    let message = server::Frame::Update(server::Update { content });
                    let json = serde_json::to_string(&message).unwrap();
                    self.sink.send(tungstenite::Message::Text(json)).await?;
                }
                SessionMessage::ClientFrame { frame, respond_to } => {
                    match frame {
                        client::Frame::Insert { char, index } => {
                            self.server.insert(self.id, LogIndex(index), char).await;
                        }
                        client::Frame::Open { url: _ } => todo!(),
                    };
                    respond_to.send(()).unwrap();
                }
                SessionMessage::Close => {
                    self.server.disconnected(self.id).await;
                }
            };
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SessionHandle {
    pub sender: acu::Sender<SessionMessage, ClientID>,
}

impl SessionHandle {
    pub async fn client_frame(&self, frame: client::Frame) {
        self.sender
            .call_with(|respond_to| SessionMessage::ClientFrame { frame, respond_to })
            .await
    }

    pub async fn update(&self, content: String) {
        self.sender.notify(SessionMessage::Update(content)).await
    }

    pub async fn close(&self) {
        self.sender.notify(SessionMessage::Close).await
    }
}

pub fn new_session<SE: std::error::Error + Send + Sync + 'static>(
    id: ClientID,
    sink: impl Sink<tungstenite::Message, Error = SE> + Unpin + Send + Sync + 'static,
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
    tokio::spawn(async move { actor.run().await.unwrap() });
    handle
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddrV4};

    const ALPHABET: &[char] = &[
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ];

    #[tokio::test]
    async fn insert() {
        let (tx, _rx) = futures::channel::mpsc::channel(8);
        let server = new_server("127.0.0.1:0");
        let client_id = ClientID::new_v4();
        let session = new_session(client_id, tx, server.clone());
        server
            .connected(
                client_id,
                session.clone(),
                SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0)),
            )
            .await;

        session
            .client_frame(client::Frame::Insert {
                char: 'a',
                index: 0,
            })
            .await;
        assert_eq!(server.get().await, "a");
    }

    #[tokio::test]
    async fn insert_many() {
        let server = new_server("127.0.0.1:0");
        let futures = (0..5).map(|_| {
            let server = server.clone();
            async move {
                let (tx, rx) = futures::channel::mpsc::channel(8);
                tokio::spawn(async move {
                    let mut rx = rx;
                    while let Some(message) = rx.next().await {
                        dbg!(message);
                    }
                });
                let id = ClientID::new_v4();
                let session = new_session(id, tx, server.clone());
                server
                    .connected(
                        id,
                        session.clone(),
                        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0)),
                    )
                    .await;
                (id, session)
            }
        });
        let sessions: Vec<_> = future::join_all(futures).await;

        for (i, (_, session)) in sessions.iter().enumerate() {
            session
                .client_frame(client::Frame::Insert { char: ALPHABET[i], index: i })
                .await;
        }

        let chars = sessions
            .iter()
            .enumerate()
            .map(|(i, _)| ALPHABET[i])
            .collect::<String>();

        let content = server.get().await;
        assert_eq!(content, chars);
    }
}
