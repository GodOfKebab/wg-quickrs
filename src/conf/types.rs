use serde::{Deserialize, Serialize};
use std::collections::HashMap;


#[allow(dead_code)]
pub(crate) enum WireGuardStatus {
    UNKNOWN,
    DOWN,
    UP,
}

impl WireGuardStatus {
    pub(crate) fn value(&self) -> u8 {
        match *self {
            WireGuardStatus::UNKNOWN => 0,
            WireGuardStatus::DOWN => 1,
            WireGuardStatus::UP => 2,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct FileConfig {
    pub(crate) agent: Agent,
    pub(crate) network: Network,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Config {
    pub(crate) agent: Agent,
    pub(crate) network: Network,
    #[serde(default)]
    pub(crate) digest: String,
    #[serde(default)]
    pub(crate) status: u8,
    #[serde(default)]
    pub(crate) timestamp: String,
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
pub(crate) struct ConfigDigest {
    #[serde(default)]
    pub(crate) digest: String,
    #[serde(default)]
    pub(crate) status: u8,
    #[serde(default)]
    pub(crate) timestamp: String,
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
pub(crate) struct Agent {
    pub(crate) address: String,
    pub(crate) web: AgentWeb,
    pub(crate) vpn: AgentVpn,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct AgentWeb {
    pub(crate) scheme: String,
    pub(crate) port: u16,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct AgentVpn {
    pub(crate) port: u16,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Network {
    pub(crate) identifier: String,
    pub(crate) subnet: String,
    pub(crate) this_peer: String,
    pub(crate) peers: HashMap<String, Peer>,
    pub(crate) connections: HashMap<String, Connection>,
    pub(crate) defaults: Defaults,
    pub(crate) leases: Vec<Lease>,
    pub(crate) updated_at: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Peer {
    pub(crate) name: String,
    pub(crate) address: String,
    pub(crate) public_key: String,
    pub(crate) private_key: String,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
    pub(crate) endpoint: EnabledValue,
    pub(crate) dns: EnabledValue,
    pub(crate) mtu: EnabledValue,
    pub(crate) scripts: Scripts,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct EnabledValue {
    pub(crate) enabled: bool,
    pub(crate) value: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Scripts {
    pub(crate) pre_up: EnabledValue,
    pub(crate) post_up: EnabledValue,
    pub(crate) pre_down: EnabledValue,
    pub(crate) post_down: EnabledValue,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Connection {
    pub(crate) enabled: bool,
    pub(crate) pre_shared_key: String,
    pub(crate) allowed_ips_a_to_b: String,
    pub(crate) allowed_ips_b_to_a: String,
    pub(crate) persistent_keepalive: EnabledValue,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Defaults {
    pub(crate) peer: DefaultPeer,
    pub(crate) connection: DefaultConnection,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct DefaultPeer {
    pub(crate) endpoint: EnabledValue,
    pub(crate) dns: EnabledValue,
    pub(crate) mtu: EnabledValue,
    pub(crate) scripts: Scripts,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct DefaultConnection {
    pub(crate) persistent_keepalive: EnabledValue,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Lease {
    pub(crate) address: String,
    pub(crate) peer_id: String,
    pub(crate) valid_until: String,
}
