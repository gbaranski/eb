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

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum Error {
    #[error("io: {0}")]
    IO(String),
    #[error("json: {0}")]
    Json(String),
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

#[async_trait]
pub trait Server {
    async fn insert(
        &self,
        params: insert::Request,
    ) -> Result<insert::Request, Error>;
    async fn open(&self, params: open::Request) -> Result<(), Error>;
}
