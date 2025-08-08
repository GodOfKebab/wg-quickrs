use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WireGuardLibError {
    #[error("types::error::peer_not_found -> peer {0} is not found")]
    PeerNotFound(String),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum WireGuardStatus {
    UNKNOWN,
    DOWN,
    UP,
}

impl WireGuardStatus {
    pub fn value(&self) -> u8 {
        match *self {
            WireGuardStatus::UNKNOWN => 0,
            WireGuardStatus::DOWN => 1,
            WireGuardStatus::UP => 2,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Config {
    pub agent: Agent,
    pub network: Network,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Summary {
    pub agent: Agent,
    pub network: Network,
    #[serde(default)]
    pub telemetry: HashMap<String, TelemetryDatum>,
    #[serde(default)]
    pub digest: String,
    #[serde(default)]
    pub status: u8,
    #[serde(default)]
    pub timestamp: String,
}

impl From<&Summary> for Config {
    fn from(config: &Summary) -> Self {
        Config {
            agent: config.agent.clone(),
            network: config.network.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SummaryDigest {
    pub telemetry: HashMap<String, TelemetryDatum>,
    pub digest: String,
    pub status: u8,
    pub timestamp: String,
}

impl From<&Summary> for SummaryDigest {
    fn from(config: &Summary) -> Self {
        SummaryDigest {
            telemetry: config.telemetry.clone(),
            digest: config.digest.clone(),
            status: config.status,
            timestamp: config.timestamp.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Agent {
    pub address: String,
    pub web: AgentWeb,
    pub vpn: AgentVpn,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AgentWeb {
    pub port: u16,
    pub use_tls: bool,
    pub password: Password,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Password {
    pub enabled: bool,
    pub hash: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AgentVpn {
    pub port: u16,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Network {
    pub identifier: String,
    pub subnet: String,
    pub this_peer: String,
    pub peers: HashMap<String, Peer>,
    pub connections: HashMap<String, Connection>,
    pub defaults: Defaults,
    pub leases: Vec<Lease>,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Peer {
    pub name: String,
    pub address: String,
    pub public_key: String,
    pub private_key: String,
    pub created_at: String,
    pub updated_at: String,
    pub endpoint: EnabledValue,
    pub dns: EnabledValue,
    pub mtu: EnabledValue,
    pub scripts: Scripts,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EnabledValue {
    pub enabled: bool,
    pub value: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Scripts {
    pub pre_up: EnabledValue,
    pub post_up: EnabledValue,
    pub pre_down: EnabledValue,
    pub post_down: EnabledValue,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Connection {
    pub enabled: bool,
    pub pre_shared_key: String,
    pub allowed_ips_a_to_b: String,
    pub allowed_ips_b_to_a: String,
    pub persistent_keepalive: EnabledValue,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Defaults {
    pub peer: DefaultPeer,
    pub connection: DefaultConnection,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct DefaultPeer {
    pub endpoint: EnabledValue,
    pub dns: EnabledValue,
    pub mtu: EnabledValue,
    pub scripts: Scripts,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct DefaultConnection {
    pub persistent_keepalive: EnabledValue,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Lease {
    pub address: String,
    pub peer_id: String,
    pub valid_until: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TelemetryDatum {
    pub latest_handshake_at: u64,
    pub transfer_a_to_b: u64,
    pub transfer_b_to_a: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct OptionalPeer {
    pub name: Option<String>,
    pub address: Option<String>,
    pub public_key: Option<String>,
    pub private_key: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub endpoint: Option<EnabledValue>,
    pub dns: Option<EnabledValue>,
    pub mtu: Option<EnabledValue>,
    pub scripts: Option<Scripts>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct OptionalConnection {
    pub enabled: Option<bool>,
    pub pre_shared_key: Option<String>,
    pub allowed_ips_a_to_b: Option<String>,
    pub allowed_ips_b_to_a: Option<String>,
    pub persistent_keepalive: Option<EnabledValue>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ChangedFields {
    pub peers: Option<HashMap<String, OptionalPeer>>,
    pub connections: Option<HashMap<String, OptionalConnection>>,
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ChangeSum {
    pub changed_fields: Option<ChangedFields>,
    pub added_peers: Option<HashMap<String, Peer>>,
    pub added_connections: Option<HashMap<String, Connection>>,
    pub removed_peers: Option<Vec<String>>,
    pub removed_connections: Option<Vec<String>>,
}

