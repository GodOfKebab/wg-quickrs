use crate::conf::network;
use crate::conf::timestamp;
use crate::WG_RUSTEZE_CONFIG_FILE;
use rust_wasm::types::{ChangeSum, ChangedFields, Config, Lease, Network, Peer};

pub(crate) use crate::conf::util::get_config;
use crate::conf::util::get_summary;
use crate::wireguard::cmd::sync_conf;
use actix_web::{web, HttpResponse};
use chrono::Duration;
use rust_wasm::validation::CheckResult;
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
    macro_rules! update_if_some {
        ($target:expr, $source:expr, $field:ident) => {
            if let Some(value) = $source.$field {
                $target.$field = value;
            }
        };
    }

    macro_rules! update_if_some_str {
        ($target:expr, $source:expr, $subtype:ident, $id:expr, $field:ident) => {
            if let Some(value) = $source.$field {
                let val_res = rust_wasm::validation::check_field(stringify!($field), &rust_wasm::validation::FieldValue {
                    str: value.clone(),
                    enabled_value: rust_wasm::types::EnabledValue {enabled: false, value: String::new()},
                });
                if !val_res.status {
                    return HttpResponse::Forbidden().json(json!({
                            "status": "forbidden",
                            "message": format!("changed_fields.{}.{}.{}: {}", stringify!($subtype), $id, stringify!($field), val_res.msg)
                        }));
                }
                $target.$field = value;
            }
        };
    }

    macro_rules! update_if_some_enabled_value {
        ($target:expr, $source:expr, $subtype:ident, $id:expr, $field:ident) => {
            if let Some(value) = $source.$field {
                let val_res = rust_wasm::validation::check_field(stringify!($field), &rust_wasm::validation::FieldValue {
                    str: String::new(),
                    enabled_value: value.clone(),
                });
                if !val_res.status {
                    return HttpResponse::Forbidden().json(json!({
                            "status": "forbidden",
                            "message": format!("changed_fields.{}.{}.{}: {}", stringify!($subtype), $id, stringify!($field), val_res.msg)
                        }));
                }
                $target.$field = value;
            }
        };
    }

    if let Some(changed_fields) = change_sum.changed_fields {
        if let Some(changed_fields_peers) = changed_fields.peers {
            for (peer_id, peer_details) in changed_fields_peers {
                if let Some(peer_config) = config.network.peers.get_mut(&peer_id) {
                    if peer_id == config.network.this_peer && peer_details.endpoint.is_some() {
                        log::info!("A client tried to change the host's endpoint! (forbidden)");
                        return HttpResponse::Forbidden().json(json!({
                            "status": "forbidden",
                            "message": "can't change the host's endpoint"
                        }));
                    }

                    update_if_some_str!(peer_config, peer_details, peer, peer_id, name);
                    update_if_some_str!(peer_config, peer_details, peer, peer_id, address);
                    update_if_some_enabled_value!(peer_config, peer_details, peer, peer_id, endpoint);
                    update_if_some_enabled_value!(peer_config, peer_details, peer, peer_id, dns);
                    update_if_some_enabled_value!(peer_config, peer_details, peer, peer_id, mtu);
                    update_if_some_str!(peer_config, peer_details, peer, peer_id, public_key);
                    update_if_some_str!(peer_config, peer_details, peer, peer_id, private_key);

                    if let Some(scripts) = peer_details.scripts {
                        update_if_some_enabled_value!(peer_config.scripts, scripts, peer, format!("{peer_id}.scripts"), pre_up);
                        update_if_some_enabled_value!(peer_config.scripts, scripts, peer, format!("{peer_id}.scripts"), post_up);
                        update_if_some_enabled_value!(peer_config.scripts, scripts, peer, format!("{peer_id}.scripts"), pre_down);
                        update_if_some_enabled_value!(peer_config.scripts, scripts, peer, format!("{peer_id}.scripts"), post_down);
                    }
                }
            }
        }
        if let Some(changed_fields_connections) = changed_fields.connections {
            for (connection_id, connection_details) in changed_fields_connections {
                if let Some(connection_config) = config.network.connections.get_mut(&connection_id) {
                    update_if_some!(connection_config, connection_details, enabled);
                    update_if_some_str!(connection_config, connection_details, connection, connection_id, pre_shared_key);
                    update_if_some_str!(connection_config, connection_details, connection, connection_id, allowed_ips_a_to_b);
                    update_if_some_str!(connection_config, connection_details, connection, connection_id, allowed_ips_b_to_a);
                    update_if_some_enabled_value!(connection_config, connection_details, connection, connection_id, persistent_keepalive);
                }
            }
        }
    }

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
