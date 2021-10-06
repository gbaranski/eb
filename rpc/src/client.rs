use serde::Serialize;
use serde::Deserialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "method", content = "params", rename_all = "camelCase")]
pub enum RequestCall {
    Insert(methods::insert::Request),
    Open(methods::open::Request)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Request {
    #[serde(rename = "jsonrpc")]
    version: String,
    id: usize,
    #[serde(flatten)]
    call: RequestCall,
}


pub mod methods {



    pub mod insert {
        use serde::Deserialize;
        use serde::Serialize;
        
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        pub struct Request {
            pub content: String,
        }
    }

    pub mod open {
        use serde::Deserialize;
        use serde::Serialize;
        use url::Url;

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        pub struct Request {
            pub url: Url,
        }
    }
    



}

#[cfg(test)]
mod tests {
    use super::Request;
    use super::methods;
    use super::RequestCall;
    use url::Url;

    #[test]
    fn open_request() {
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
                call: RequestCall::Open(methods::open::Request{
                    url: Url::parse("file:///project-a/main.rs").unwrap(),
                })
            }
        );
    }
}
