use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub(crate) const CONF_FILE: &str = ".wg-rusteze/conf.yml";


#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct Config { network: Network }

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Network {
    subnet: String,
    peers: HashMap<String, Peer>,
    connections: HashMap<String, Connection>,
    defaults: Defaults,
    #[serde(default)]
    status: i8,
    #[serde(default)]
    timestamp: i64,
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

