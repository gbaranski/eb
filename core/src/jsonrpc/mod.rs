mod error;

pub use error::Error;
pub use error::ErrorCode;
use std::borrow::Cow;
use serde::Deserialize;
use serde::Serialize;
use serde::Deserializer;
use serde::de;
use serde::Serializer;

/// An incoming JSON-RPC message.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Response(Response),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Request {
    pub jsonrpc: Version,
    pub method: Cow<'static, str>,
    #[serde(flatten)]
    pub kind: RequestKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestKind {
    Request { params: serde_json::Value, id: Id },
    Notification { params: serde_json::Value },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Response {
    jsonrpc: Version,
    pub id: Id,
    #[serde(flatten)]
    pub kind: ResponseKind,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum ResponseKind {
    Ok { result: serde_json::Value },
    Err { error: Error },
}


/// A unique ID used to correlate requests and responses together.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Id {
    /// Numeric ID.
    Number(i64),
    /// String ID.
    String(String),
    /// Null ID.
    ///
    /// While `null` is considered a valid request ID by the JSON-RPC 2.0 specification, its use is
    /// _strongly_ discouraged because the specification also uses a `null` value to indicate an
    /// unknown ID in the [`Response`] object.
    Null,
}

impl Default for Id {
    fn default() -> Self {
        Id::Null
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Id::Number(id) => std::fmt::Display::fmt(id, f),
            Id::String(id) => std::fmt::Debug::fmt(id, f),
            Id::Null => f.write_str("null"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Version;

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Cow::<'de, str>::deserialize(deserializer)?.as_ref() {
            "2.0" => Ok(Version),
            _ => Err(de::Error::custom("expected JSON-RPC version \"2.0\"")),
        }
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        "2.0".serialize(serializer)
    }
}
