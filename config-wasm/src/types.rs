use serde::{Deserialize, Serialize};
use std::collections::HashMap;


#[allow(dead_code)]
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
pub struct FileConfig {
    pub agent: Agent,
    pub network: Network,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Config {
    pub agent: Agent,
    pub network: Network,
    #[serde(default)]
    pub digest: String,
    #[serde(default)]
    pub status: u8,
    #[serde(default)]
    pub timestamp: String,
}

impl From<&Config> for FileConfig {
    fn from(config: &Config) -> Self {
        FileConfig {
            agent: config.agent.clone(),
            network: config.network.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ConfigDigest {
    #[serde(default)]
    pub digest: String,
    #[serde(default)]
    pub status: u8,
    #[serde(default)]
    pub timestamp: String,
}

impl From<&Config> for ConfigDigest {
    fn from(config: &Config) -> Self {
        ConfigDigest {
            digest: config.digest.clone(),
            status: config.status.clone(),
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
    pub scheme: String,
    pub port: u16,
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
