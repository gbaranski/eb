use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "method", content = "params", rename_all = "camelCase")]
pub enum Call {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Request {
    #[serde(rename = "jsonrpc")]
    version: String,
    id: usize,
    #[serde(flatten)]
    call: Call,
}

pub mod methods {}

#[cfg(test)]
mod tests {
    use super::methods;
    use super::Call;
    use super::Request;
    use url::Url;

    #[test]
    fn open() {
        const JSON: &str = r#"
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "open",
            "params": {
                "url": "file:///project-a/main.rs"
            }

        }
        "#;
        assert_eq!(
            serde_json::from_str::<Request>(JSON).unwrap(),
            Request {
                version: "2.0".to_string(),
                id: 1,
                call: Call::Open(methods::Open {
                    url: Url::parse("file:///project-a/main.rs").unwrap(),
                })
            }
        );
    }
}
