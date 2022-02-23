pub type ClientID = uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::Display)]
pub enum Author {
    Server,
    Client(ClientID),
}

pub type Chronofold = chronofold::Chronofold<Author, char>;
pub type Change = chronofold::Change<char>;
pub type Op = chronofold::Op<Author, char>;

pub mod client {
    use serde::Deserialize;
    use serde::Serialize;
    use url::Url;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "kebab-case")]
    pub enum Frame {
        Insert { char: char, index: usize },
        Open { url: Url },
    }
}

pub mod server {
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "kebab-case")]
    pub enum Frame {
        Update(Update),
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Update {
        pub content: String,
    }
}
