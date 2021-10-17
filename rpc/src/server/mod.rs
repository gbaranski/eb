pub mod ws;

use serde::Deserialize;
use serde::Serialize;

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

use async_trait::async_trait;
use crate::Error;

#[async_trait]
pub trait Peer {
    async fn insert(&self, params: insert::Request) -> Result<insert::Response, Error>;
    async fn open(&self, params: open::Request) -> Result<open::Response, Error>;
}
