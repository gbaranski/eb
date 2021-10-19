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
    server.run("127.0.0.1:8080").await?;
    Ok(())
}

use tokio::net::ToSocketAddrs;
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

    pub async fn run(self, address: impl ToSocketAddrs) -> Result<(), anyhow::Error> {
        eb_rpc::server::run(address).await?;
        Ok(())
    }
}