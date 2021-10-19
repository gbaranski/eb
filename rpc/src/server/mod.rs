use crate::Error;
use futures::StreamExt;
use serde::Deserialize;
use serde::Serialize;
use tokio::net::TcpListener;
use tokio::net::ToSocketAddrs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Insert(insert::Request),
    Open(open::Request),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Insert(insert::Response),
    Open(open::Response),
}

pub mod insert {
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Request {
        pub content: String,
        pub cursor: usize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Response {}
}

pub mod open {
    use serde::Deserialize;
    use serde::Serialize;
    use url::Url;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Request {
        pub url: Url,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Response {}
}

pub async fn run(address: impl ToSocketAddrs) -> Result<(), Error> {
    let listener = TcpListener::bind(address).await?;
    loop {
        let (stream, address) = listener.accept().await?;
        tokio::spawn(async move {
            let stream = tokio_tungstenite::accept_async(stream).await.unwrap();
            let (tx, rx) = stream.split();

        });
    }
}
