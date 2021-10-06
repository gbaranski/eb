const LOG_ENV: &str = "EB_SERVER_LOG";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    use std::str::FromStr;
    use tracing_subscriber::EnvFilter;

    let env_filter = match std::env::var(LOG_ENV) {
        Ok(env) => env,
        Err(std::env::VarError::NotPresent) => "info".to_string(),
        Err(std::env::VarError::NotUnicode(_)) => panic!(
            "{} environment variable is not valid unicode and can't be read",
            LOG_ENV
        ),
    };
    let env_filter = EnvFilter::from_str(&env_filter)
        .unwrap_or_else(|err| panic!("invalid `{}` environment variable {}", LOG_ENV, err));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
    let server = Server::new();
    eb_rpc::run_ws(server, "127.0.0.1:8080").await?;
    Ok(())
}

use async_trait::async_trait;
use tokio::sync::Mutex;

struct Server {
    content: Mutex<String>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            content: Mutex::new(String::new()),
        }
    }
}

#[async_trait]
impl eb_rpc::Server for Server {
    async fn insert(&self, params: eb_rpc::client::insert::Request) -> Result<(), eb_rpc::Error> {
        tracing::info!("insert. params = {:?}", params);
        self.content
            .lock()
            .await
            .insert_str(params.cursor, &params.content);
        Ok(())
    }

    async fn open(&self, params: eb_rpc::client::open::Request) -> Result<(), eb_rpc::Error> {
        tracing::info!("open. params = {:?}", params);
        Ok(())
    }
}
