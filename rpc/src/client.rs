use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Insert(insert::Request),
    Open(open::Request),
}

pub mod insert {
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Request {
        pub content: String,
        pub cursor: usize,
    }
}

pub mod open {
    use serde::Deserialize;
    use serde::Serialize;
    use url::Url;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Request {
        pub url: Url,
    }
}
