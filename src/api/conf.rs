use serde::{Deserialize, Serialize};
use serde_yml::Serializer;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::io::Write;
use std::ops::Deref;
use std::time::SystemTime;

pub(crate) const DEFAULT_CONF_FILE: &str = ".wg-rusteze/conf.yml";


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
    pub(crate) network_digest: String,
    #[serde(default)]
    pub(crate) status: u8,
    #[serde(default)]
    pub(crate) timestamp: u64,
}

impl Config {
    pub(crate) fn put_timestamp(&mut self) -> &mut Self {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => self.timestamp = n.as_secs(),
            Err(_) => self.timestamp = 0,
        }
        return self;
    }
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
    pub(crate) network_digest: String,
    #[serde(default)]
    pub(crate) status: u8,
    #[serde(default)]
    pub(crate) timestamp: u64,
}

impl From<&Config> for ConfigDigest {
    fn from(config: &Config) -> Self {
        ConfigDigest {
            network_digest: config.network_digest.clone(),
            status: config.status.clone(),
            timestamp: config.timestamp.clone(),
        }
    }
}


impl ConfigDigest {
    pub(crate) fn put_timestamp(&mut self) -> &mut Self {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => self.timestamp = n.as_secs(),
            Err(_) => self.timestamp = 0,
        }
        return self;
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
struct Network {
    subnet: String,
    this_peer: String,
    peers: HashMap<String, Peer>,
    connections: HashMap<String, Connection>,
    defaults: Defaults,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Peer {
    name: String,
    address: String,
    public_key: String,
    private_key: String,
    created_at: String,
    updated_at: String,
    mobility: String,
    endpoint: String,
    dns: EnabledValue,
    mtu: EnabledValue,
    scripts: Scripts
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct EnabledValue { enabled: bool, value: String}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Scripts { pre_up: EnabledValue, post_up: EnabledValue, pre_down: EnabledValue, post_down: EnabledValue }

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Connection {
    enabled: bool,
    pre_shared_key: String,
    allowed_ips_a_to_b: String,
    allowed_ips_b_to_a: String,
    persistent_keepalive: EnabledValue,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Defaults {
    peer: DefaultPeer,
    connection: DefaultConnection,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct DefaultPeer {
    dns: EnabledValue,
    mtu: EnabledValue,
    scripts: Scripts,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct DefaultConnection {
    persistent_keepalive: EnabledValue,
}

pub(crate) fn get_config() -> Config {
    let file_contents = fs::read_to_string(DEFAULT_CONF_FILE).expect("Unable to open file");
    let mut config: Config = serde_yml::from_str(&file_contents).unwrap();

    // Make sure agent fields get precedence over network fields
    if config.network.peers.get(&config.network.this_peer).unwrap().endpoint != format!("{}:{}", config.agent.address, config.agent.vpn.port) {
        config.network.peers.get_mut(&config.network.this_peer).unwrap().endpoint = format!("{}:{}", config.agent.address, config.agent.vpn.port);
        set_config(&config);
    }

    let mut buf = [0u8; 64];
    let network_digest: &str = base16ct::lower::encode_str(&Sha256::digest(file_contents.as_bytes()), &mut buf).expect("Unable to calculate network digest");
    config.network_digest = network_digest.to_string();
    config.status = WireGuardStatus::UP.value(); // TODO: replace
    config.put_timestamp();

    return config;
}

pub(crate) fn set_config(config: &Config) {
    let file_config = FileConfig::from(config);
    let config_str = serde_yml::to_string(&file_config).expect("Failed to serialize config");

    let mut file = File::create(DEFAULT_CONF_FILE).expect("Failed to open config file");
    file.write_all(config_str.as_bytes()).expect("Failed to write to config file");
    log::info!("Updated config file")
}
