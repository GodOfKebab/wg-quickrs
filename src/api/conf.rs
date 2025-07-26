use actix_web::HttpResponse;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::time::SystemTime;

pub(crate) const DEFAULT_CONF_FILE: &str = ".wg-rusteze/conf.yml";

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
    identifier: String,
    subnet: String,
    this_peer: String,
    peers: HashMap<String, Peer>,
    connections: HashMap<String, Connection>,
    defaults: Defaults,
    updated_at: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Peer {
    name: String,
    address: String,
    public_key: String,
    private_key: String,
    created_at: String,
    updated_at: String,
    endpoint: EnabledValue,
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
    if config.network.peers.get(&config.network.this_peer).unwrap().endpoint.value != format!("{}:{}", config.agent.address, config.agent.vpn.port) {
        config.network.peers.get_mut(&config.network.this_peer).unwrap().endpoint.value = format!("{}:{}", config.agent.address, config.agent.vpn.port);
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

pub(crate) fn update_config(change_sum: Value) -> HttpResponse {
    // Open the config file for reading and writing
    let mut config_file_reader = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(DEFAULT_CONF_FILE)
        .expect("Failed to open config file");

    // Read the existing contents
    let mut config_str = String::new();
    config_file_reader.read_to_string(&mut config_str).expect("Failed to read config file");
    let config_file: FileConfig = serde_yml::from_str(&config_str).unwrap();
    let mut config_value: Value = match serde_yml::from_str(&config_str) {
        Ok(val) => val,
        Err(_err) => {
            return HttpResponse::NotFound()
                .content_type("application/json")
                .body(r#"{"status":"forbidden","message":"Unable to parse config file"}"#);
        }
    };

    let network_config = match config_value.get_mut("network") {
        Some(n) => n,
        None => return HttpResponse::NotFound()
            .content_type("application/json")
            .body(r#"{"status":"forbidden","message":"Unable to parse config file"}"#),
    };

    // process changed_fields
    if let Some(changed_fields) = change_sum.get("changed_fields") {
        {
            if changed_fields
                .get("peers")
                .and_then(|p| p.as_object())
                .and_then(|peers| peers.get(config_file.network.this_peer.as_str()))
                .and_then(|this_peer| this_peer.get("endpoint"))
                .is_some()
            {
                log::info!("A client tried to change the host's endpoint! (forbidden)");
                return HttpResponse::Forbidden()
                    .content_type("application/json")
                    .body(r#"{"status":"forbidden","message":"can't change the host's endpoint"}"#);
            }
        }
        { apply_changes(network_config, "peers", changed_fields); }
        { apply_changes(network_config, "connections", changed_fields); }
    }

    // process added_connections
    if let Some(added_connections) = change_sum.get("added_connections") {
        if let Some(added_connections_map) = added_connections.as_object() {
            for (connection_id, connection_details) in added_connections_map {
                {
                    if let Some(connections) = network_config.get_mut("connections") {
                        connections[connection_id] = connection_details.clone();
                    }
                }
            }
        }
    }

    // process removed_connections
    if let Some(removed_connections) = change_sum.get("removed_connections") {
        if let Some(removed_connections_map) = removed_connections.as_object() {
            for (connection_id, _connection_details) in removed_connections_map {
                {
                    if let Some(connections) = network_config.get_mut("connections") {
                        if let Some(connections_map) = connections.as_object_mut() {
                            connections_map.remove(connection_id);
                        }
                    }
                }
            }
        }
    }


    let config_str = serde_yml::to_string(&config_value)
        .expect("Failed to serialize config");

    // Move back to beginning and truncate before writing
    config_file_reader.set_len(0).expect("Failed to truncate config file");
    config_file_reader.seek(SeekFrom::Start(0)).expect("Failed to seek to start");
    config_file_reader.write_all(config_str.as_bytes()).expect("Failed to write to config file");

    log::info!("Updated config file");
    return HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"status":"ok"}"#) // ✅ sends JSON response
}

fn apply_changes(network_config: &mut Value, section_name: &str, changed_fields: &Value) {
    if let Some(config_section) = network_config.get_mut(section_name) {
        if let Some(section) = changed_fields.get(section_name) {
            if let Some(section_map) = section.as_object() {
                for (item_id, item_changes) in section_map {
                    let item_config = match config_section.get_mut(item_id) {
                        Some(cfg) => cfg,
                        None => continue,
                    };

                    let item_changes_map = match item_changes.as_object() {
                        Some(map) => map,
                        None => continue,
                    };

                    for (field_key, field_value) in item_changes_map {
                        item_config[field_key] = field_value.clone();
                    }
                }
            }
        }
    }
}

