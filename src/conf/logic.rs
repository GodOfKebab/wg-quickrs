use crate::conf::network;
use crate::conf::timestamp;
use crate::conf::types::{Config, FileConfig, Lease, WireGuardStatus};
use crate::WG_RUSTEZE_CONFIG_FILE;

use crate::conf;
use actix_web::{web, HttpResponse};
use chrono::Duration;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use uuid::Uuid;

pub(crate) fn respond_get_network_summary(params: web::Query<crate::api::SummaryBody>) -> HttpResponse {
    let resp_body = if params.only_digest {
        serde_json::to_string(&conf::types::ConfigDigest::from(&get_config()))
    } else {
        serde_json::to_string(&get_config())
    }.unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(resp_body)
}

pub(crate) fn get_config() -> Config {
    let file_contents = fs::read_to_string(WG_RUSTEZE_CONFIG_FILE.get().expect("WG_RUSTEZE_CONFIG_FILE not set")).expect("Unable to open file");
    let mut config: Config = serde_yml::from_str(&file_contents).unwrap();

    // Make sure agent fields get precedence over network fields
    if config.network.peers.get(&config.network.this_peer).unwrap().endpoint.value != format!("{}:{}", config.agent.address, config.agent.vpn.port) {
        log::info!("detected mismatch between configured wg-rusteze agent endpoints and wireguard peer endpoints! overriding wireguard peer endpoints");
        config.network.peers.get_mut(&config.network.this_peer).unwrap().endpoint.value = format!("{}:{}", config.agent.address, config.agent.vpn.port);
        set_config(&config);
    }

    let mut buf = [0u8; 64];
    let digest: &str = base16ct::lower::encode_str(&Sha256::digest(file_contents.as_bytes()), &mut buf).expect("Unable to calculate network digest");
    config.digest = digest.to_string();
    config.status = WireGuardStatus::UP.value(); // TODO: replace
    config.timestamp = timestamp::get_now_timestamp_formatted();

    return config;
}

pub(crate) fn set_config(config: &Config) {
    let mut file_config = FileConfig::from(config);
    file_config.network.updated_at = timestamp::get_now_timestamp_formatted();
    let config_str = serde_yml::to_string(&file_config).expect("Failed to serialize config");

    let mut file = File::create(WG_RUSTEZE_CONFIG_FILE.get().expect("WG_RUSTEZE_CONFIG_FILE not set")).expect("Failed to open config file");
    file.write_all(config_str.as_bytes()).expect("Failed to write to config file");
    log::info!("Updated config file")
}

pub(crate) fn respond_patch_network_config(body: web::Bytes) -> HttpResponse {
    let body_raw = String::from_utf8_lossy(&body);
    let change_sum: Value = match serde_json::from_str(&body_raw) {
        Ok(val) => val,
        Err(err) => {
            return HttpResponse::BadRequest()
                .content_type("application/json")
                .body(format!(r#"{{"error":"Invalid JSON: {}"}}"#, err));
        }
    };

    log::info!("update_config with the change_sum = {}", change_sum);
    // Open the config file for reading and writing
    let mut config_file_reader = OpenOptions::new()
        .read(true)
        .write(true)
        .open(WG_RUSTEZE_CONFIG_FILE.get().expect("WG_RUSTEZE_CONFIG_FILE not set"))
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

    // TODO: process errors

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

    // process added_peers
    if let Some(added_peers) = change_sum.get("added_peers") {
        if let Some(added_peers_map) = added_peers.as_object() {
            for (peer_id, peer_details) in added_peers_map {
                {
                    if let Some(peers) = network_config.get_mut("peers") {
                        peers[peer_id] = peer_details.clone();
                        peers[peer_id]["created_at"] = Value::String(timestamp::get_now_timestamp_formatted());
                        peers[peer_id]["updated_at"] = peers[peer_id]["created_at"].clone();
                    }
                    // remove leased id/address
                    if let Some(leases_array) = network_config
                        .get_mut("leases")
                        .and_then(|v| v.as_array_mut()) {
                        leases_array.retain(|lease| lease.get("peer_id").and_then(|v| v.as_str()) != Some(peer_id));
                    }
                }
            }
        }
    }

    // process removed_peers
    if let Some(removed_peers) = change_sum.get("removed_peers") {
        if let Some(removed_peers_map) = removed_peers.as_object() {
            for (peer_id, _peer_details) in removed_peers_map {
                {
                    if let Some(peers) = network_config.get_mut("peers") {
                        if let Some(peers_map) = peers.as_object_mut() {
                            peers_map.remove(peer_id);
                        }
                    }
                }
            }
        }
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

    if let Some(updated_at) = network_config.get_mut("updated_at") {
        *updated_at = Value::String(timestamp::get_now_timestamp_formatted());
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
        .body(r#"{"status":"ok"}"#)
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
                        if field_key.eq("scripts") {
                            if let Some(scripts_map) = field_value.as_object() {
                                for (script_key, script_value) in scripts_map {
                                    let scripts_config = match item_config.get_mut("scripts") {
                                        Some(cfg) => cfg,
                                        None => continue,
                                    };
                                    scripts_config[script_key] = script_value.clone();
                                }
                            }
                        } else {
                            item_config[field_key] = field_value.clone();
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn respond_get_network_lease_id_address() -> HttpResponse {
    // Open the config file for reading and writing
    let mut config_file_reader = OpenOptions::new()
        .read(true)
        .write(true)
        .open(WG_RUSTEZE_CONFIG_FILE.get().expect("WG_RUSTEZE_CONFIG_FILE not set"))
        .expect("Failed to open config file");

    // Read the existing contents
    let mut config_str = String::new();
    config_file_reader.read_to_string(&mut config_str).expect("Failed to read config file");
    let mut config_file: FileConfig = serde_yml::from_str(&config_str).unwrap();

    let mut reserved_addresses = Vec::<String>::new();
    for peer in config_file.network.peers.values() {
        reserved_addresses.push(peer.address.clone());
    }
    config_file.network.leases.retain(|lease| {
        timestamp::get_duration_since_formatted(lease.valid_until.clone()) > Duration::zero()
    });

    for lease in config_file.network.leases.clone() {
        reserved_addresses.push(lease.address.clone());
    }
    let next_address = network::get_next_available_address(&config_file.network.subnet, &reserved_addresses);
    log::info!("Next available address: {:?}", next_address);

    let body = Lease {
        address: next_address.unwrap(),
        peer_id: String::from(Uuid::new_v4()),
        valid_until: timestamp::get_future_timestamp_formatted(Duration::minutes(10)),
    };
    config_file.network.leases.push(body.clone());
    config_file.network.updated_at = timestamp::get_now_timestamp_formatted();
    let config_str = serde_yml::to_string(&config_file).expect("Failed to serialize config");
    // Move back to beginning and truncate before writing
    config_file_reader.set_len(0).expect("Failed to truncate config file");
    config_file_reader.seek(SeekFrom::Start(0)).expect("Failed to seek to start");
    config_file_reader.write_all(config_str.as_bytes()).expect("Failed to write to config file");

    return HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&body).unwrap())
}
