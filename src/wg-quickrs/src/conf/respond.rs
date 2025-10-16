use crate::macros::*;
use crate::conf::util;
use crate::conf::network;
use crate::conf::timestamp;
use crate::wireguard::cmd::sync_conf;
use wg_quickrs_wasm::types::*;
use wg_quickrs_wasm::validation::*;
use actix_web::{HttpResponse, web};
use chrono::Duration;
use serde_json::json;
use uuid::Uuid;


pub(crate) fn get_network_summary(query: web::Query<crate::web::api::SummaryBody>) -> HttpResponse {
    let summary = match util::get_summary() {
        Ok(summary) => summary,
        Err(_) => {
            return HttpResponse::InternalServerError().body("Unable to get config");
        }
    };
    let response_data = if query.only_digest {
        json!(wg_quickrs_wasm::types::SummaryDigest::from(&summary))
    } else {
        json!(summary)
    };
    HttpResponse::Ok().json(response_data)
}

macro_rules! get_mg_config_w_digest {
    () => {{
        let mut_opt = util::CONFIG_W_NETWORK_DIGEST.get();
        if mut_opt.is_none() {
            return HttpResponse::InternalServerError().json(json!({
                "status": "internal_server_error",
                "message": "can't handle request: internal config variables are not initialized"
            }));
        }

        let mut_lock_opt = mut_opt.unwrap().lock();
        if mut_lock_opt.is_err() {
            return HttpResponse::InternalServerError().json(json!({
                "status": "internal_server_error",
                "message": "can't handle request: unable to acquire lock on config variables"
            }));
        }

        mut_lock_opt.unwrap()
    }};
}

macro_rules! post_mg_config_w_digest {
    ($c:expr) => {
        let file_config = ConfigFile{
            version: wg_quickrs_version!().into(),
            agent: $c.agent.clone(),
            network: $c.network_w_digest.network.clone(),
        };
        $c.network_w_digest.network.updated_at = timestamp::get_now_timestamp_formatted();
        $c.network_w_digest = match util::NetworkWDigest::from_network($c.network_w_digest.network.clone()) {
            Ok(network_digest) => network_digest,
            Err(_) => {
                return HttpResponse::InternalServerError().json(json!({
                    "status": "internal_server_error",
                    "message": "can't handle request: unable to compute config digest"
                }));
            }
        };
        let file_config_str = match serde_yml::to_string(&file_config).map_err(util::ConfUtilError::Serialization) {
            Ok(s) => s,
            Err(_) => {
                return HttpResponse::InternalServerError().json(json!({
                    "status": "internal_server_error",
                    "message": "can't handle request: unable to serialize config"
                }));
            }
        };
        match util::write_config(file_config_str) {
            Ok(_) => {}
            Err(_) => {
                return HttpResponse::InternalServerError().json(json!({
                    "status": "internal_server_error",
                    "message": "can't handle request: unable to write config"
                }));
            }
        }
        log::info!("updated config file");
    };
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

    // process changed_fields
    macro_rules! update_bool {
        ($target:expr, $source:expr, $field:ident) => {
            if let Some(value) = $source.$field {
                $target.$field = value;
            }
        };
    }

    macro_rules! validate_return_400 {
        ($val_res:expr, $field_parent:expr, $field:ident) => {
            if !$val_res.status {
                return HttpResponse::BadRequest().json(json!({
                    "status": "bad_request",
                    "message": format!("{}.{}: {}", $field_parent, stringify!($field), $val_res.msg)
                }));
            }
        }
    }

    macro_rules! validate_str {
        ($value:expr, $field_parent:expr, $field:ident) => {
            let val_res = check_field_str(stringify!($field), $value);
            validate_return_400!(val_res, $field_parent, $field);
        };
    }

    macro_rules! validate_enabled_value {
        ($value:expr, $field_parent:expr, $field:ident) => {
            let val_res = check_field_enabled_value(stringify!($field), $value);
            validate_return_400!(val_res, $field_parent, $field);
        };
    }

    macro_rules! validate_then_update_str {
        ($target:expr, $source:expr, $subtype:ident, $id:expr, $field:ident) => {
            if let Some(value) = $source.$field {
                validate_str!(
                    &value,
                    format!("changed_fields.{}.{}", stringify!($subtype), $id),
                    $field
                );
                $target.$field = value;
            }
        };
    }

    macro_rules! validate_then_update_enabled_value {
        ($target:expr, $source:expr, $subtype:ident, $id:expr, $field:ident) => {
            if let Some(value) = $source.$field {
                validate_enabled_value!(
                    &value,
                    format!("changed_fields.{}.{}", stringify!($subtype), $id),
                    $field
                );
                $target.$field = value;
            }
        };
    }

    macro_rules! validate_then_update_script {
        ($target:expr, $source:expr, $id:expr, $field:ident) => {
            if let Some(enabled_values) = $source.$field {
                $target.scripts.$field = Vec::new();
                for (i, enabled_value) in enabled_values.iter().enumerate() {
                    validate_enabled_value!(
                        enabled_value,
                        format!("changed_fields.peer.{}.scripts[{}]", $id, i),
                        $field
                    );

                    if !enabled_value.enabled {
                        $target.scripts.$field.push(enabled_value.clone());
                        continue;
                    }
                    for script_string_line in
                        enabled_value.value.split(";").filter(|&x| !x.is_empty())
                    {
                        $target.scripts.$field.push(EnabledValue {
                            enabled: true,
                            value: format!("{script_string_line};"),
                        })
                    }
                }
            }
        };
    }

    let mut c = get_mg_config_w_digest!();
    let this_peer = c.network_w_digest.network.this_peer.clone();
    let mut changed_config = false;

    if let Some(changed_fields) = change_sum.changed_fields {
        if let Some(changed_fields_peers) = changed_fields.peers {
            for (peer_id, peer_details) in changed_fields_peers {
                if let Some(peer_config) = c.network_w_digest.network.peers.get_mut(&peer_id) {
                    if peer_id == this_peer && peer_details.endpoint.is_some() {
                        log::info!("A client tried to change the host's endpoint! (forbidden)");
                        return HttpResponse::Forbidden().json(json!({
                            "status": "forbidden",
                            "message": "can't change the host's endpoint"
                        }));
                    }

                    validate_then_update_str!(peer_config, peer_details, peer, peer_id, name);
                    validate_then_update_str!(peer_config, peer_details, peer, peer_id, address);
                    validate_then_update_enabled_value!(
                        peer_config,
                        peer_details,
                        peer,
                        peer_id,
                        endpoint
                    );
                    validate_then_update_str!(peer_config, peer_details, peer, peer_id, kind);
                    validate_then_update_enabled_value!(
                        peer_config,
                        peer_details,
                        peer,
                        peer_id,
                        icon
                    );
                    validate_then_update_enabled_value!(
                        peer_config,
                        peer_details,
                        peer,
                        peer_id,
                        dns
                    );
                    validate_then_update_enabled_value!(
                        peer_config,
                        peer_details,
                        peer,
                        peer_id,
                        mtu
                    );
                    validate_then_update_str!(peer_config, peer_details, peer, peer_id, public_key);
                    validate_then_update_str!(
                        peer_config,
                        peer_details,
                        peer,
                        peer_id,
                        private_key
                    );

                    if let Some(scripts) = peer_details.scripts {
                        validate_then_update_script!(peer_config, scripts, peer_id, pre_up);
                        validate_then_update_script!(peer_config, scripts, peer_id, post_up);
                        validate_then_update_script!(peer_config, scripts, peer_id, pre_down);
                        validate_then_update_script!(peer_config, scripts, peer_id, post_down);
                    }
                    changed_config = true;
                } else {
                    return HttpResponse::NotFound().json(json!({
                        "status": "not_found",
                        "message": format!("peer '{peer_id}' does not exist")
                    }));
                }
            }
        }
        if let Some(changed_fields_connections) = changed_fields.connections {
            for (connection_id, connection_details) in changed_fields_connections {
                if let Some(connection_config) =
                    c.network_w_digest.network.connections.get_mut(&connection_id)
                {
                    update_bool!(connection_config, connection_details, enabled);
                    validate_then_update_str!(
                        connection_config,
                        connection_details,
                        connection,
                        connection_id,
                        pre_shared_key
                    );
                    validate_then_update_str!(
                        connection_config,
                        connection_details,
                        connection,
                        connection_id,
                        allowed_ips_a_to_b
                    );
                    validate_then_update_str!(
                        connection_config,
                        connection_details,
                        connection,
                        connection_id,
                        allowed_ips_b_to_a
                    );
                    validate_then_update_enabled_value!(
                        connection_config,
                        connection_details,
                        connection,
                        connection_id,
                        persistent_keepalive
                    );
                    changed_config = true;
                } else {
                    return HttpResponse::NotFound().json(json!({
                        "status": "not_found",
                        "message": format!("connection '{connection_id}' does not exist")
                    }));
                }
            }
        }
    }

    // process added_peers
    if let Some(added_peers) = change_sum.added_peers {
        for (peer_id, peer_details) in added_peers {
            {
                validate_str!(&peer_details.name, format!("added_peers.{}", peer_id), name);
                validate_str!(
                    &peer_details.address,
                    format!("added_peers.{}", peer_id),
                    address
                );
                validate_enabled_value!(
                    &peer_details.endpoint,
                    format!("added_peers.{}", peer_id),
                    endpoint
                );
                validate_str!(&peer_details.kind, format!("added_peers.{}", peer_id), kind);
                validate_enabled_value!(
                    &peer_details.icon,
                    format!("added_peers.{}", peer_id),
                    icon
                );
                validate_enabled_value!(&peer_details.dns, format!("added_peers.{}", peer_id), dns);
                validate_enabled_value!(&peer_details.mtu, format!("added_peers.{}", peer_id), mtu);
                validate_str!(
                    &peer_details.public_key,
                    format!("added_peers.{}", peer_id),
                    public_key
                );
                validate_str!(
                    &peer_details.private_key,
                    format!("added_peers.{}", peer_id),
                    private_key
                );
                for (i, enabled_value) in peer_details.scripts.pre_up.iter().enumerate() {
                    validate_enabled_value!(
                        enabled_value,
                        format!("added_peers.{}.scripts.pre_up[{}]", peer_id, i),
                        pre_up
                    );
                }
                for (i, enabled_value) in peer_details.scripts.post_up.iter().enumerate() {
                    validate_enabled_value!(
                        enabled_value,
                        format!("added_peers.{}.scripts.post_up[{}]", peer_id, i),
                        post_up
                    );
                }
                for (i, enabled_value) in peer_details.scripts.pre_down.iter().enumerate() {
                    validate_enabled_value!(
                        enabled_value,
                        format!("added_peers.{}.scripts.pre_down[{}]", peer_id, i),
                        pre_down
                    );
                }
                for (i, enabled_value) in peer_details.scripts.post_down.iter().enumerate() {
                    validate_enabled_value!(
                        enabled_value,
                        format!("added_peers.{}.scripts.post_down[{}]", peer_id, i),
                        post_down
                    );
                }
                let mut added_peer = wg_quickrs_wasm::types::Peer::from(&peer_details);
                added_peer.created_at = timestamp::get_now_timestamp_formatted();
                added_peer.updated_at = added_peer.created_at.clone();
                c.network_w_digest.network.peers.insert(peer_id.clone(), added_peer);
                // remove the new peer id/address from the lease
                c.network_w_digest
                    .network
                    .leases
                    .retain(|lease| lease.peer_id != peer_id);
                changed_config = true;
            }
        }
    }

    // process removed_peers
    if let Some(removed_peers) = change_sum.removed_peers {
        for peer_id in removed_peers {
            {
                c.network_w_digest.network.peers.remove(peer_id.as_str());
                changed_config = true;
            }
        }
    }

    // process added_connections
    if let Some(added_connections) = change_sum.added_connections {
        for (connection_id, connection_details) in added_connections {
            {
                validate_str!(
                    &connection_details.pre_shared_key,
                    format!("added_connections.{}", connection_id),
                    pre_shared_key
                );
                validate_str!(
                    &connection_details.allowed_ips_a_to_b,
                    format!("added_connections.{}", connection_id),
                    allowed_ips_a_to_b
                );
                validate_str!(
                    &connection_details.allowed_ips_b_to_a,
                    format!("added_connections.{}", connection_id),
                    allowed_ips_b_to_a
                );
                validate_enabled_value!(
                    &connection_details.persistent_keepalive,
                    format!("added_connections.{}", connection_id),
                    persistent_keepalive
                );
                c.network_w_digest
                    .network
                    .connections
                    .insert(connection_id.clone(), connection_details);
                changed_config = true;
            }
        }
    }

    // process removed_connections
    if let Some(removed_connections) = change_sum.removed_connections {
        for connection_id in removed_connections {
            {
                c.network_w_digest.network.connections.remove(connection_id.as_str());
                changed_config = true;
            }
        }
    }
    if !changed_config {
        log::info!("nothing to update");
        return HttpResponse::BadRequest().json(json!({
            "status": "bad_request",
            "message": "nothing to update"
        }));
    }
    post_mg_config_w_digest!(c);

    if c.agent.vpn.enabled {
        match sync_conf(&c.clone().to_config()) {
            Ok(_) => {}
            Err(e) => {
                log::error!("{e}");
                return HttpResponse::InternalServerError().into();
            }
        };
        log::info!("synchronized config file");
    }

    HttpResponse::Ok().json(json!({
        "status": "ok"
    }))
}

pub(crate) fn get_network_lease_id_address() -> HttpResponse {
    let mut c = get_mg_config_w_digest!();

    let mut reserved_addresses = Vec::<String>::new();
    for peer in c.network_w_digest.network.peers.values() {
        reserved_addresses.push(peer.address.clone());
    }
    c.network_w_digest.network.leases.retain(|lease| {
        timestamp::get_duration_since_formatted(lease.valid_until.clone()) < Duration::zero()
    });
    for lease in c.network_w_digest.network.leases.clone() {
        reserved_addresses.push(lease.address.clone());
    }
    let next_address =
        match network::get_next_available_address(&c.network_w_digest.network.subnet, &reserved_addresses) {
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
    c.network_w_digest.network.leases.push(body.clone());
    post_mg_config_w_digest!(c);

    HttpResponse::Ok().json(json!(body))
}
