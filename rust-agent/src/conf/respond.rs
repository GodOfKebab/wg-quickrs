use crate::conf::network;
use crate::conf::timestamp;
use crate::WG_RUSTEZE_CONFIG_FILE;
use rust_wasm::types::{ChangeSum, Config, Lease};

pub(crate) use crate::conf::util::get_config;
use crate::conf::util::get_summary;
use crate::wireguard::cmd::sync_conf;
use actix_web::{web, HttpResponse};
use chrono::Duration;
use serde_json::{json, Value};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use uuid::Uuid;

pub(crate) fn get_network_summary(query: web::Query<crate::web::api::SummaryBody>) -> HttpResponse {
    let summary = match get_summary() {
        Ok(summary) => summary,
        Err(_) => {
            return HttpResponse::InternalServerError().body("Unable to get config");
        }
    };
    let response_data = if query.only_digest {
        json!(rust_wasm::types::SummaryDigest::from(&summary))
    } else {
        json!(summary)
    };
    HttpResponse::Ok().json(response_data)
}

pub(crate) fn patch_network_config(body: web::Bytes) -> HttpResponse {
    let body_raw = String::from_utf8_lossy(&body);
    let change_sum: ChangeSum = match serde_json::from_str(&body_raw) {
        Ok(val) => val,
        Err(err) => {
            return HttpResponse::BadRequest().json(json!({
                "error": format!("Invalid JSON: {err}")
            }));
        }
    };

    log::info!("update_config with the change_sum = \n{:?}", change_sum);
    // Open the config file for reading and writing
    let mut config_file_reader = OpenOptions::new()
        .read(true)
        .write(true)
        .open(
            WG_RUSTEZE_CONFIG_FILE
                .get()
                .expect("WG_RUSTEZE_CONFIG_FILE not set"),
        )
        .expect("Failed to open config file");

    // Read the existing contents
    let mut config_str = String::new();
    config_file_reader
        .read_to_string(&mut config_str)
        .expect("Failed to read config file");
    let mut config: Config = serde_yml::from_str(&config_str).unwrap();
    // TODO: process errors

    // process changed_fields
    // if let Some(changed_fields) = change_sum.get("changed_fields") {
    //     {
    //         if changed_fields
    //             .get("peers")
    //             .and_then(|p| p.as_object())
    //             .and_then(|peers| peers.get(config_file.network.this_peer.as_str()))
    //             .and_then(|this_peer| this_peer.get("endpoint"))
    //             .is_some()
    //         {
    //             log::info!("A client tried to change the host's endpoint! (forbidden)");
    //             return HttpResponse::Forbidden().json(json!({
    //                 "status": "forbidden",
    //                 "message": "can't change the host's endpoint"
    //             }));
    //         }
    //     }
    //     {
    //         apply_changes(network_config, "peers", changed_fields);
    //     }
    //     {
    //         apply_changes(network_config, "connections", changed_fields);
    //     }
    // }

    // process added_peers
    if let Some(added_peers) = change_sum.added_peers {
        for (peer_id, peer_details) in added_peers {
            {
                config.network.peers.insert(peer_id.clone(), peer_details);
                // remove leased id/address
                config.network.leases.retain(|lease| { lease.peer_id != peer_id });
            }
        }
    }

    // process removed_peers
    if let Some(removed_peers) = change_sum.removed_peers {
        for peer_id in removed_peers {
            {
                config.network.peers.remove(peer_id.as_str());
            }
        }
    }

    // process added_connections
    if let Some(added_connections) = change_sum.added_connections {
        for (connection_id, connection_details) in added_connections {
            {
                config.network.connections.insert(
                    connection_id.clone(),
                    connection_details,
                );
            }
        }
    }

    // process removed_connections
    if let Some(removed_connections) = change_sum.removed_connections {
        for connection_id in removed_connections {
            {
                config.network.connections.remove(connection_id.as_str());
            }
        }
    }

    config.network.updated_at = timestamp::get_now_timestamp_formatted();

    let config_str = serde_yml::to_string(&config).expect("Failed to serialize config");

    // Move back to the beginning and truncate before writing
    config_file_reader
        .set_len(0)
        .expect("Failed to truncate config file");
    config_file_reader
        .seek(SeekFrom::Start(0))
        .expect("Failed to seek to start");
    config_file_reader
        .write_all(config_str.as_bytes())
        .expect("Failed to write to config file");
    log::info!("updated config file");

    match sync_conf(&config) {
        Ok(_) => {}
        Err(e) => {
            log::error!("{e}");
            return HttpResponse::InternalServerError().into();
        }
    };
    log::info!("synchronized config file");

    HttpResponse::Ok().json(json!({
        "status": "ok"
    }))
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

pub(crate) fn get_network_lease_id_address() -> HttpResponse {
    // Open the config file for reading and writing
    let mut config_file_reader = match OpenOptions::new()
        .read(true)
        .write(true)
        .open(WG_RUSTEZE_CONFIG_FILE.get().unwrap())
    {
        Ok(file) => file,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Failed to open config file".to_string());
        }
    };

    // Read the existing contents
    let mut file_contents = String::new();
    match config_file_reader.read_to_string(&mut file_contents) {
        Ok(_) => {}
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Failed to read config file".to_string());
        }
    }
    let mut config: Config = match serde_yml::from_str(&file_contents) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Failed to read config file".to_string());
        }
    };

    let mut reserved_addresses = Vec::<String>::new();
    for peer in config.network.peers.values() {
        reserved_addresses.push(peer.address.clone());
    }
    config.network.leases.retain(|lease| {
        timestamp::get_duration_since_formatted(lease.valid_until.clone()) > Duration::zero()
    });

    for lease in config.network.leases.clone() {
        reserved_addresses.push(lease.address.clone());
    }
    let next_address =
        match network::get_next_available_address(&config.network.subnet, &reserved_addresses) {
            Some(next_address) => next_address,
            None => {
                return HttpResponse::InternalServerError()
                    .body("Failed to get next available address".to_string());
            }
        };

    let body = Lease {
        address: next_address,
        peer_id: String::from(Uuid::new_v4()),
        valid_until: timestamp::get_future_timestamp_formatted(Duration::minutes(10)),
    };
    log::info!("leased address {} until {}", body.address, body.valid_until);
    config.network.leases.push(body.clone());
    config.network.updated_at = timestamp::get_now_timestamp_formatted();
    file_contents = match serde_yml::to_string(&config) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Failed to serialize config".to_string());
        }
    };

    // Move back to the beginning and truncate before writing
    match config_file_reader.set_len(0) {
        Ok(_) => {}
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Failed to truncate config file".to_string());
        }
    }
    match config_file_reader.seek(SeekFrom::Start(0)) {
        Ok(_) => {}
        Err(_) => {
            return HttpResponse::InternalServerError().body("Failed to seek to start".to_string());
        }
    }
    match config_file_reader.write_all(file_contents.as_bytes()) {
        Ok(_) => {}
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Failed to write to config file".to_string());
        }
    }
    HttpResponse::Ok().json(json!(body))
}
