use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;


#[derive(Error, Debug)]
pub enum WireGuardLibError {
    #[error("types::error::peer_not_found -> peer {0} is not found")]
    PeerNotFound(Uuid),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum WireGuardStatus {
    UNKNOWN,
    DOWN,
    UP,
}