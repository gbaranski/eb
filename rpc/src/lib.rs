pub mod client;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io: {0}")]
    IO(#[from] std::io::Error),
    #[error("json: {0}")]
    JSON(#[from] serde_json::Error),
}