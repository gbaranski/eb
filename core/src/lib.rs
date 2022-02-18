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

pub type SessionID = uuid::Uuid;

pub mod client {
    use serde::Deserialize;
    use serde::Serialize;
    use url::Url;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[non_exhaustive]
    #[serde(tag = "type", rename_all = "kebab-case")]
    pub enum Frame {
        Insert(Insert),
        Open(Open),
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Insert {
        pub cursor: usize,
        pub content: String,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Open {
        pub url: Url,
    }
}

pub mod server {
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[non_exhaustive]
    #[serde(tag = "type", rename_all = "kebab-case")]
    pub enum Frame {
        Update,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Update {
        pub content: String,
    }
}
