use serde::Deserialize;
use crate::Position;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Update{
        start: Position,
        end: Position,
        text: String,
    }
}

