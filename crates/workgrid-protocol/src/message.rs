use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("invalid message: {0}")]
    InvalidMessage(String),
    #[error("auth failed: {0}")]
    AuthFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlMessage {
    Register { server_id: String, public_key: String },
    PairRequest { server_id: String },
    PairAck { server_id: String },
}

impl ControlMessage {
    pub fn register(server_id: String, public_key: String) -> Self {
        Self::Register { server_id, public_key }
    }

    pub fn pair_request(server_id: String) -> Self {
        Self::PairRequest { server_id }
    }

    pub fn pair_ack(server_id: String) -> Self {
        Self::PairAck { server_id }
    }

    pub fn server_id(&self) -> Option<&String> {
        match self {
            ControlMessage::Register { server_id, .. }
            | ControlMessage::PairRequest { server_id }
            | ControlMessage::PairAck { server_id } => Some(server_id),
        }
    }
}
