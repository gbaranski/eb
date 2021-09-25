pub mod client;
pub mod server;

use serde::Serialize;
use serde::Deserialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}


pub const DEFAULT_TCP_PORT: u16 = 65432;
