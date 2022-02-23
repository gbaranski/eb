use eb_server::new_server;

const LOG_ENV: &str = "EB_LOG";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    use std::str::FromStr;
    use tracing::Level;

    let level = match std::env::var(LOG_ENV) {
        Ok(env) => Level::from_str(&env)?,
        Err(std::env::VarError::NotPresent) => Level::INFO,
        Err(std::env::VarError::NotUnicode(_)) => panic!(
            "{} environment variable is not valid unicode and can't be read",
            LOG_ENV
        ),
    };
    tracing_subscriber::fmt().with_max_level(level).init();
    let server = new_server("127.0.0.1:8080");
    server.sender.closed().await;
    Ok(())
}
