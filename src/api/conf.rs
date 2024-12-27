use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct Config {
    network: Network,
    #[serde(default)]
    status: u8,
    #[serde(default)]
    timestamp: u64,
}

impl Config {
    pub(crate) fn set_status(&mut self, status: u8) -> &mut Self {
        self.status = status;
        return self;
    }

    pub(crate) fn put_timestamp(&mut self) -> &mut Self {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => self.timestamp = n.as_secs(),
            Err(_) => self.timestamp = 0,
        }
        return self;
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Network {
    subnet: String,
    this_peer: String,
    peers: HashMap<String, Peer>,
    connections: HashMap<String, Connection>,
    defaults: Defaults,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Peer {
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct EnabledValue { enabled: bool, value: String}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Scripts { pre_up: EnabledValue, post_up: EnabledValue, pre_down: EnabledValue, post_down: EnabledValue }

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Connection {
    enabled: bool,
    pre_shared_key: String,
    allowed_ips_a_to_b: String,
    allowed_ips_b_to_a: String,
    persistent_keepalive: EnabledValue,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Defaults {
    peer: DefaultPeer,
    connection: DefaultConnection,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct DefaultPeer {
    dns: EnabledValue,
    mtu: EnabledValue,
    scripts: Scripts,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct DefaultConnection {
    persistent_keepalive: EnabledValue,
}

