use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WireGuardLibError {
    #[error("types::error::peer_not_found -> peer {0} is not found")]
    PeerNotFound(String),
    #[error("types::error::key_decode_failed -> {0}")]
    KeyDecodeFailed(String),
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
    pub network: Network,
    pub telemetry: Option<Telemetry>,
    pub digest: String,
    pub status: u8,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SummaryDigest {
    pub telemetry: Option<Telemetry>,
    pub digest: String,
    pub status: u8,
    pub timestamp: String,
}

impl From<&Summary> for SummaryDigest {
    fn from(summary: &Summary) -> Self {
        SummaryDigest {
            telemetry: summary.telemetry.clone(),
            digest: summary.digest.clone(),
            status: summary.status,
            timestamp: summary.timestamp.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Agent {
    pub web: AgentWeb,
    pub vpn: AgentVpn,
    pub firewall: AgentFirewall,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AgentWeb {
    pub address: String,
    pub http: AgentWebHttp,
    pub https: AgentWebHttps,
    pub password: Password,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AgentWebHttp {
    pub enabled: bool,
    pub port: u16,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AgentWebHttps {
    pub enabled: bool,
    pub port: u16,
    pub tls_cert: PathBuf,
    pub tls_key: PathBuf,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Password {
    pub enabled: bool,
    pub hash: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AgentVpn {
    pub enabled: bool,
    pub port: u16,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AgentFirewall {
    pub enabled: bool,
    pub utility: PathBuf,
    pub gateway: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Network {
    pub identifier: String,
    pub subnet: String,
    pub this_peer: String,
    pub peers: HashMap<String, Peer>,
    pub connections: HashMap<String, Connection>,
    pub defaults: Defaults,
    pub reservations: HashMap<String, ReservationData>,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Peer {
    pub name: String,
    pub address: String,
    pub endpoint: EnabledValue,
    pub kind: String,
    pub icon: EnabledValue,
    pub dns: EnabledValue,
    pub mtu: EnabledValue,
    pub scripts: Scripts,
    pub private_key: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EnabledValue {
    pub enabled: bool,
    pub value: String,
}

impl Default for EnabledValue {
    fn default() -> Self {
        EnabledValue {
            enabled: false,
            value: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Scripts {
    pub pre_up: Vec<EnabledValue>,
    pub post_up: Vec<EnabledValue>,
    pub pre_down: Vec<EnabledValue>,
    pub post_down: Vec<EnabledValue>,
}

impl Default for Scripts {
    fn default() -> Self {
        Scripts {
            pre_up: vec![],
            post_up: vec![],
            pre_down: vec![],
            post_down: vec![],
        }
    }
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

impl Default for Defaults {
    fn default() -> Self {
        Defaults {
            peer: Default::default(),
            connection: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct DefaultPeer {
    pub endpoint: EnabledValue,
    pub kind: String,
    pub icon: EnabledValue,
    pub dns: EnabledValue,
    pub mtu: EnabledValue,
    pub scripts: Scripts,
}

impl Default for DefaultPeer {
    fn default() -> Self {
        DefaultPeer {
            endpoint: Default::default(),
            kind: "".to_string(),
            icon: Default::default(),
            dns: Default::default(),
            mtu: Default::default(),
            scripts: Default::default(),
        }
    }
}


#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct DefaultConnection {
    pub persistent_keepalive: EnabledValue,
}

impl Default for DefaultConnection {
    fn default() -> Self {
        DefaultConnection {
            persistent_keepalive: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ReservationData {
    pub peer_id: String,
    pub valid_until: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Telemetry {
    pub max_len: u8,
    pub data: Vec<TelemetryData>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TelemetryData {
    pub datum: HashMap<String, TelemetryDatum>,
    pub timestamp: u128,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TelemetryDatum {
    pub latest_handshake_at: u64,
    pub transfer_a_to_b: u64,
    pub transfer_b_to_a: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct OptionalScripts {
    pub pre_up: Option<Vec<EnabledValue>>,
    pub post_up: Option<Vec<EnabledValue>>,
    pub pre_down: Option<Vec<EnabledValue>>,
    pub post_down: Option<Vec<EnabledValue>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct OptionalPeer {
    pub name: Option<String>,
    pub address: Option<String>,
    pub endpoint: Option<EnabledValue>,
    pub kind: Option<String>,
    pub icon: Option<EnabledValue>,
    pub dns: Option<EnabledValue>,
    pub mtu: Option<EnabledValue>,
    pub scripts: Option<OptionalScripts>,
    pub private_key: Option<String>,
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
pub struct AddedPeer {
    pub name: String,
    pub address: String,
    pub endpoint: EnabledValue,
    pub kind: String,
    pub icon: EnabledValue,
    pub dns: EnabledValue,
    pub mtu: EnabledValue,
    pub scripts: Scripts,
    pub private_key: String,
}

impl From<&AddedPeer> for Peer {
    fn from(added_peer: &AddedPeer) -> Self {
        Peer {
            name: added_peer.name.clone(),
            address: added_peer.address.clone(),
            endpoint: added_peer.endpoint.clone(),
            kind: added_peer.kind.clone(),
            icon: added_peer.icon.clone(),
            dns: added_peer.dns.clone(),
            mtu: added_peer.mtu.clone(),
            scripts: added_peer.scripts.clone(),
            private_key: added_peer.private_key.clone(),
            created_at: "".to_string(),
            updated_at: "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ChangeSum {
    pub changed_fields: Option<ChangedFields>,
    pub added_peers: Option<HashMap<String, AddedPeer>>,
    pub added_connections: Option<HashMap<String, Connection>>,
    pub removed_peers: Option<Vec<String>>,
    pub removed_connections: Option<Vec<String>>,
}
