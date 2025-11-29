use crate::conf::util;
use crate::conf::network;
use crate::wireguard::cmd::sync_conf;
use wg_quickrs_lib::types::api::{SummaryDigest, ChangeSum};
use wg_quickrs_lib::validation::network::{*, validate_amnezia_s1, validate_amnezia_s2, validate_amnezia_s1_s2, validate_amnezia_jc, validate_amnezia_jmin, validate_amnezia_jmax, validate_amnezia_jmin_jmax};
use actix_web::{HttpResponse, web};
use chrono::{Duration, Utc};
use serde_json::json;
use uuid::Uuid;
use wg_quickrs_lib::helpers::remove_expired_reservations;
use wg_quickrs_lib::types::network::{ReservationData, NetworkWDigest};
use wg_quickrs_lib::types::config::ConfigFile;

macro_rules! get_mg_config_w_digest {
    () => {{
        util::CONFIG_W_NETWORK_DIGEST
            .get()
            .ok_or_else(|| HttpResponse::InternalServerError().body("internal config variables are not initialized"))?
            .write()
            .map_err(|_| HttpResponse::InternalServerError().body("unable to acquire lock on config variables"))?
    }};
}

macro_rules! post_mg_config_w_digest {
    ($c:expr) => {{
        let config_file = ConfigFile::from(&$c.to_config());
        $c.network_w_digest.network.updated_at = Utc::now();
        $c.network_w_digest = NetworkWDigest::try_from($c.network_w_digest.network.clone())
            .map_err(|_| HttpResponse::InternalServerError().body("unable to compute config digest"))?;

        let config_file_str = serde_yml::to_string(&config_file)
            .map_err(|_| HttpResponse::InternalServerError().body("unable to serialize config"))?;

        util::write_config(config_file_str)
            .map_err(|_| HttpResponse::InternalServerError().body("unable to write config"))?;
    }};
}

pub(crate) fn get_network_summary(query: web::Query<crate::web::api::SummaryBody>) -> Result<HttpResponse, HttpResponse> {
    let summary = util::get_summary()
        .map_err(|_| HttpResponse::InternalServerError().body("unable to get summary"))?;
    let response_data = if query.only_digest {
        json!(SummaryDigest::from(&summary))
    } else {
        json!(summary)
    };
    Ok(HttpResponse::Ok().json(response_data))
}

pub(crate) fn patch_network_config(body: web::Bytes) -> Result<HttpResponse, HttpResponse> {
    let body_raw = String::from_utf8_lossy(&body);
    let change_sum: ChangeSum = match serde_json::from_str(&body_raw) {
        Ok(val) => val,
        Err(err) => {
            return Err(HttpResponse::BadRequest().body(format!("invalid JSON: {}", err)));
        }
    };

    log::debug!("update config with the change_sum = \n{:?}", change_sum);

    let mut c = get_mg_config_w_digest!();
    let this_peer_id = c.network_w_digest.network.this_peer;
    let mut changed_config = false;

    remove_expired_reservations(&mut c.network_w_digest.network);

    // process changed_fields
    if let Some(changed_fields) = &change_sum.changed_fields {
        if let Some(changed_fields_peers) = &changed_fields.peers {
            for (peer_id, peer_details) in changed_fields_peers {
                let mut network_copy = c.network_w_digest.network.clone();
                if let Some(peer_config) = c.network_w_digest.network.peers.get_mut(peer_id) {
                    if let Some(name) = &peer_details.name {
                        peer_config.name = parse_and_validate_peer_name(name).map_err(|e| {
                            HttpResponse::BadRequest().body(format!("changed_fields.peers.{}.name: {}", peer_id, e))
                        })?;
                    }
                    if let Some(address) = &peer_details.address {
                        network_copy.peers.retain(|id, _| id != peer_id);
                        peer_config.address = validate_peer_address(address, &network_copy).map_err(|e| {
                            HttpResponse::BadRequest().body(format!("changed_fields.peers.{}.address: {}", peer_id, e))
                        })?;
                    }
                    if let Some(endpoint) = &peer_details.endpoint {
                        peer_config.endpoint = validate_peer_endpoint(endpoint).map_err(|e| {
                            HttpResponse::BadRequest().body(format!("changed_fields.peers.{}.endpoint: {}", peer_id, e))
                        })?;
                    }
                    if let Some(kind) = &peer_details.kind {
                        peer_config.kind = parse_and_validate_peer_kind(kind).map_err(|e| {
                            HttpResponse::BadRequest().body(format!("changed_fields.peers.{}.kind: {}", peer_id, e))
                        })?;
                    }
                    if let Some(icon) = &peer_details.icon {
                        peer_config.icon = validate_peer_icon(icon).map_err(|e| {
                            HttpResponse::BadRequest().body(format!("changed_fields.peers.{}.icon: {}", peer_id, e))
                        })?;
                    }
                    if let Some(dns) = &peer_details.dns {
                        peer_config.dns = validate_peer_dns(dns).map_err(|e| {
                            HttpResponse::BadRequest().body(format!("changed_fields.peers.{}.dns: {}", peer_id, e))
                        })?;
                    }
                    if let Some(mtu) = &peer_details.mtu {
                        peer_config.mtu = validate_peer_mtu(mtu).map_err(|e| {
                            HttpResponse::BadRequest().body(format!("changed_fields.peers.{}.mtu: {}", peer_id, e))
                        })?;
                    }
                    if let Some(private_key) = &peer_details.private_key {
                        peer_config.private_key = *private_key;
                        // If deserialization succeeds, private_key is already validated.
                    }

                    if let Some(scripts) = &peer_details.scripts {
                        // Security check: prevent modifying scripts for this_peer
                        if *peer_id == this_peer_id {
                            return Err(HttpResponse::Forbidden().body("cannot modify scripts for this peer remotely"));
                        }

                        if let Some(scripts) = &scripts.pre_up {
                            peer_config.scripts.pre_up = validate_peer_scripts(scripts).map_err(|e| {
                                HttpResponse::BadRequest().body(format!("changed_fields.peers.{}.scripts.pre_up: {}", peer_id, e))
                            })?;
                        }
                        if let Some(scripts) = &scripts.post_up {
                            peer_config.scripts.post_up = validate_peer_scripts(scripts).map_err(|e| {
                                HttpResponse::BadRequest().body(format!("changed_fields.peers.{}.scripts.post_up: {}", peer_id, e))
                            })?;
                        }
                        if let Some(scripts) = &scripts.pre_down {
                            peer_config.scripts.pre_down = validate_peer_scripts(scripts).map_err(|e| {
                                HttpResponse::BadRequest().body(format!("changed_fields.peers.{}.scripts.pre_down: {}", peer_id, e))
                            })?;
                        }
                        if let Some(scripts) = &scripts.post_down {
                            peer_config.scripts.post_down = validate_peer_scripts(scripts).map_err(|e| {
                                HttpResponse::BadRequest().body(format!("changed_fields.peers.{}.scripts.post_down: {}", peer_id, e))
                            })?;
                        }
                    }

                    if let Some(amnezia_parameters) = &peer_details.amnezia_parameters {
                        if let Some(jc) = amnezia_parameters.jc {
                            validate_amnezia_jc(jc).map_err(|e| {
                                HttpResponse::BadRequest().body(format!("changed_fields.peers.{}.amnezia_parameters.jc: {}", peer_id, e))
                            })?;
                            peer_config.amnezia_parameters.jc = jc;
                        }
                        if let Some(jmin) = amnezia_parameters.jmin {
                            validate_amnezia_jmin(jmin).map_err(|e| {
                                HttpResponse::BadRequest().body(format!("changed_fields.peers.{}.amnezia_parameters.jmin: {}", peer_id, e))
                            })?;
                            peer_config.amnezia_parameters.jmin = jmin;
                        }
                        if let Some(jmax) = amnezia_parameters.jmax {
                            validate_amnezia_jmax(jmax).map_err(|e| {
                                HttpResponse::BadRequest().body(format!("changed_fields.peers.{}.amnezia_parameters.jmax: {}", peer_id, e))
                            })?;
                            peer_config.amnezia_parameters.jmax = jmax;
                        }
                        // Validate jmin and jmax relationship if either is present
                        if amnezia_parameters.jmin.is_some() || amnezia_parameters.jmax.is_some() {
                            validate_amnezia_jmin_jmax(
                                peer_config.amnezia_parameters.jmin,
                                peer_config.amnezia_parameters.jmax
                            ).map_err(|e| {
                                HttpResponse::BadRequest().body(format!("changed_fields.peers.{}.amnezia_parameters: {}", peer_id, e))
                            })?;
                        }
                    }
                    changed_config = true;
                } else {
                    return Err(HttpResponse::NotFound().body(format!("peer '{}' does not exist", peer_id)));
                }
            }
        }
        if let Some(changed_fields_connections) = &changed_fields.connections {
            for (connection_id, connection_details) in changed_fields_connections {
                if let Some(connection_config) =
                    c.network_w_digest.network.connections.get_mut(connection_id)
                {
                    if let Some(enabled) = connection_details.enabled {
                        connection_config.enabled = enabled;
                    }
                    if let Some(pre_shared_key) = connection_details.pre_shared_key {
                        connection_config.pre_shared_key = pre_shared_key;
                        // If deserialization succeeds, pre_shared_key is already validated.
                    }
                    if let Some(allowed_ips_a_to_b) = &connection_details.allowed_ips_a_to_b {
                        connection_config.allowed_ips_a_to_b = allowed_ips_a_to_b.clone();
                        // If deserialization succeeds, allowed_ips_a_to_b is already validated.
                    }
                    if let Some(allowed_ips_b_to_a) = &connection_details.allowed_ips_b_to_a {
                        connection_config.allowed_ips_b_to_a = allowed_ips_b_to_a.clone();
                        // If deserialization succeeds, allowed_ips_b_to_a is already validated.
                    }
                    if let Some(persistent_keepalive) = &connection_details.persistent_keepalive {
                        connection_config.persistent_keepalive = validate_conn_persistent_keepalive(persistent_keepalive).map_err(|e| {
                            HttpResponse::BadRequest().body(format!("changed_fields.connections.{}.persistent_keepalive: {}", connection_id, e))
                        })?;
                    }
                    changed_config = true;
                } else {
                    return Err(HttpResponse::NotFound().body(format!("connection '{}' does not exist", connection_id)));
                }
            }
        }
        if let Some(changed_fields_network) = &changed_fields.network {
            if let Some(amnezia_parameters) = &changed_fields_network.amnezia_parameters {
                if let Some(enabled) = amnezia_parameters.enabled {
                    c.network_w_digest.network.amnezia_parameters.enabled = enabled;
                }
                if let Some(s1) = amnezia_parameters.s1 {
                    validate_amnezia_s1(s1).map_err(|e| {
                        HttpResponse::BadRequest().body(format!("changed_fields.network.amnezia_parameters.s1: {}", e))
                    })?;
                    c.network_w_digest.network.amnezia_parameters.s1 = s1;
                }
                if let Some(s2) = amnezia_parameters.s2 {
                    validate_amnezia_s2(s2).map_err(|e| {
                        HttpResponse::BadRequest().body(format!("changed_fields.network.amnezia_parameters.s2: {}", e))
                    })?;
                    c.network_w_digest.network.amnezia_parameters.s2 = s2;
                }
                // Validate s1 and s2 relationship if both are present
                if amnezia_parameters.s1.is_some() || amnezia_parameters.s2.is_some() {
                    validate_amnezia_s1_s2(
                        c.network_w_digest.network.amnezia_parameters.s1,
                        c.network_w_digest.network.amnezia_parameters.s2
                    ).map_err(|e| {
                        HttpResponse::BadRequest().body(format!("changed_fields.network.amnezia_parameters: {}", e))
                    })?;
                }
                if let Some(h1) = amnezia_parameters.h1 {
                    c.network_w_digest.network.amnezia_parameters.h1 = h1;
                }
                if let Some(h2) = amnezia_parameters.h2 {
                    c.network_w_digest.network.amnezia_parameters.h2 = h2;
                }
                if let Some(h3) = amnezia_parameters.h3 {
                    c.network_w_digest.network.amnezia_parameters.h3 = h3;
                }
                if let Some(h4) = amnezia_parameters.h4 {
                    c.network_w_digest.network.amnezia_parameters.h4 = h4;
                }
                changed_config = true;
            }
        }
        if let Some(changed_fields_defaults) = &changed_fields.defaults {
            if let Some(default_peer) = &changed_fields_defaults.peer {
                if let Some(kind) = &default_peer.kind {
                    c.network_w_digest.network.defaults.peer.kind = parse_and_validate_peer_kind(kind).map_err(|e| {
                        HttpResponse::BadRequest().body(format!("changed_fields.defaults.peer.kind: {}", e))
                    })?;
                }
                if let Some(icon) = &default_peer.icon {
                    c.network_w_digest.network.defaults.peer.icon = validate_peer_icon(icon).map_err(|e| {
                        HttpResponse::BadRequest().body(format!("changed_fields.defaults.peer.icon: {}", e))
                    })?;
                }
                if let Some(dns) = &default_peer.dns {
                    c.network_w_digest.network.defaults.peer.dns = validate_peer_dns(dns).map_err(|e| {
                        HttpResponse::BadRequest().body(format!("changed_fields.defaults.peer.dns: {}", e))
                    })?;
                }
                if let Some(mtu) = &default_peer.mtu {
                    c.network_w_digest.network.defaults.peer.mtu = validate_peer_mtu(mtu).map_err(|e| {
                        HttpResponse::BadRequest().body(format!("changed_fields.defaults.peer.mtu: {}", e))
                    })?;
                }
                if let Some(scripts) = &default_peer.scripts {
                    if let Some(pre_up) = &scripts.pre_up {
                        c.network_w_digest.network.defaults.peer.scripts.pre_up = validate_peer_scripts(pre_up).map_err(|e| {
                            HttpResponse::BadRequest().body(format!("changed_fields.defaults.peer.scripts.pre_up: {}", e))
                        })?;
                    }
                    if let Some(post_up) = &scripts.post_up {
                        c.network_w_digest.network.defaults.peer.scripts.post_up = validate_peer_scripts(post_up).map_err(|e| {
                            HttpResponse::BadRequest().body(format!("changed_fields.defaults.peer.scripts.post_up: {}", e))
                        })?;
                    }
                    if let Some(pre_down) = &scripts.pre_down {
                        c.network_w_digest.network.defaults.peer.scripts.pre_down = validate_peer_scripts(pre_down).map_err(|e| {
                            HttpResponse::BadRequest().body(format!("changed_fields.defaults.peer.scripts.pre_down: {}", e))
                        })?;
                    }
                    if let Some(post_down) = &scripts.post_down {
                        c.network_w_digest.network.defaults.peer.scripts.post_down = validate_peer_scripts(post_down).map_err(|e| {
                            HttpResponse::BadRequest().body(format!("changed_fields.defaults.peer.scripts.post_down: {}", e))
                        })?;
                    }
                }
                if let Some(amnezia_parameters) = &default_peer.amnezia_parameters {
                    if let Some(jc) = amnezia_parameters.jc {
                        validate_amnezia_jc(jc).map_err(|e| {
                            HttpResponse::BadRequest().body(format!("changed_fields.defaults.peer.amnezia_parameters.jc: {}", e))
                        })?;
                        c.network_w_digest.network.defaults.peer.amnezia_parameters.jc = jc;
                    }
                    if let Some(jmin) = amnezia_parameters.jmin {
                        validate_amnezia_jmin(jmin).map_err(|e| {
                            HttpResponse::BadRequest().body(format!("changed_fields.defaults.peer.amnezia_parameters.jmin: {}", e))
                        })?;
                        c.network_w_digest.network.defaults.peer.amnezia_parameters.jmin = jmin;
                    }
                    if let Some(jmax) = amnezia_parameters.jmax {
                        validate_amnezia_jmax(jmax).map_err(|e| {
                            HttpResponse::BadRequest().body(format!("changed_fields.defaults.peer.amnezia_parameters.jmax: {}", e))
                        })?;
                        c.network_w_digest.network.defaults.peer.amnezia_parameters.jmax = jmax;
                    }
                    // Validate jmin and jmax relationship if either is present
                    if amnezia_parameters.jmin.is_some() || amnezia_parameters.jmax.is_some() {
                        validate_amnezia_jmin_jmax(
                            c.network_w_digest.network.defaults.peer.amnezia_parameters.jmin,
                            c.network_w_digest.network.defaults.peer.amnezia_parameters.jmax
                        ).map_err(|e| {
                            HttpResponse::BadRequest().body(format!("changed_fields.defaults.peer.amnezia_parameters: {}", e))
                        })?;
                    }
                }
                changed_config = true;
            }
            if let Some(default_connection) = &changed_fields_defaults.connection {
                if let Some(persistent_keepalive) = &default_connection.persistent_keepalive {
                    c.network_w_digest.network.defaults.connection.persistent_keepalive = validate_conn_persistent_keepalive(persistent_keepalive).map_err(|e| {
                        HttpResponse::BadRequest().body(format!("changed_fields.defaults.connection.persistent_keepalive: {}", e))
                    })?;
                }
                changed_config = true;
            }
        }
    }

    // process added_peers
    if let Some(added_peers) = &change_sum.added_peers {
        for (peer_id, peer_details) in added_peers {
            {
                if c.network_w_digest.network.peers.contains_key(peer_id) {
                    return Err(HttpResponse::Forbidden().body(format!("peer '{}' already exists", peer_id)));
                }
                if let Some(value) = c.network_w_digest.network.reservations.get(&peer_details.address)
                    && value.peer_id != *peer_id {
                    return Err(HttpResponse::Forbidden().body(format!("address '{}' is reserved for another peer_id", peer_details.address)));
                }
                // ensure the address is taken off the reservation list so check_internal_address succeeds (this won't be posted if it fails early)
                c.network_w_digest
                    .network
                    .reservations
                    .retain(|address, _|  *address != peer_details.address);

                // If deserialization succeeds, peer_id is already validated.
                parse_and_validate_peer_name(&peer_details.name).map_err(|e| {
                    HttpResponse::BadRequest().body(format!("added_peers.{}.name: {}", peer_id, e))
                })?;
                validate_peer_address(&peer_details.address, &c.network_w_digest.network).map_err(|e| {
                    HttpResponse::BadRequest().body(format!("added_peers.{}.address: {}", peer_id, e))
                })?;
                validate_peer_endpoint(&peer_details.endpoint).map_err(|e| {
                    HttpResponse::BadRequest().body(format!("added_peers.{}.endpoint: {}", peer_id, e))
                })?;
                parse_and_validate_peer_kind(&peer_details.kind).map_err(|e| {
                    HttpResponse::BadRequest().body(format!("added_peers.{}.kind: {}", peer_id, e))
                })?;
                validate_peer_icon(&peer_details.icon).map_err(|e| {
                    HttpResponse::BadRequest().body(format!("added_peers.{}.icon: {}", peer_id, e))
                })?;
                validate_peer_dns(&peer_details.dns).map_err(|e| {
                    HttpResponse::BadRequest().body(format!("added_peers.{}.dns: {}", peer_id, e))
                })?;
                // If deserialization succeeds, dns is already validated.
                validate_peer_mtu(&peer_details.mtu).map_err(|e| {
                    HttpResponse::BadRequest().body(format!("added_peers.{}.mtu: {}", peer_id, e))
                })?;
                // If deserialization succeeds, private_key is already validated.
                validate_peer_scripts(&peer_details.scripts.pre_up).map_err(|e| {
                    HttpResponse::BadRequest().body(format!("added_peers.{}.scripts.pre_up: {}", peer_id, e))
                })?;
                validate_peer_scripts(&peer_details.scripts.post_up).map_err(|e| {
                    HttpResponse::BadRequest().body(format!("added_peers.{}.scripts.post_up: {}", peer_id, e))
                })?;
                validate_peer_scripts(&peer_details.scripts.pre_down).map_err(|e| {
                    HttpResponse::BadRequest().body(format!("added_peers.{}.scripts.pre_down: {}", peer_id, e))
                })?;
                validate_peer_scripts(&peer_details.scripts.post_down).map_err(|e| {
                    HttpResponse::BadRequest().body(format!("added_peers.{}.scripts.post_down: {}", peer_id, e))
                })?;
                let mut added_peer = wg_quickrs_lib::types::network::Peer::from(peer_details);
                added_peer.created_at = Utc::now();
                added_peer.updated_at = added_peer.created_at;
                c.network_w_digest.network.peers.insert(*peer_id, added_peer);
                changed_config = true;
            }
        }
    }

    // process removed_peers
    if let Some(removed_peers) = &change_sum.removed_peers {
        for peer_id in removed_peers {
            {
                if *peer_id == this_peer_id {
                    return Err(HttpResponse::Forbidden().body("cannot remove this peer"));
                }
                c.network_w_digest.network.peers.remove(peer_id);
                // automatically remove connections
                for connection_id in c.network_w_digest.network.connections.clone().keys().filter(|&x| x.contains(peer_id)) {
                    c.network_w_digest.network.connections.remove(connection_id);
                }
                changed_config = true;
            }
        }
    }

    // process added_connections
    if let Some(added_connections) = &change_sum.added_connections {
        for (connection_id, connection_details) in added_connections {
            {
                if !c.network_w_digest.network.peers.contains_key(&connection_id.a) {
                    return Err(HttpResponse::BadRequest().body(format!("added_connections.{}: 'peer_id' does not exist", connection_id.a)));
                }
                if !c.network_w_digest.network.peers.contains_key(&connection_id.b) {
                    return Err(HttpResponse::BadRequest().body(format!("added_connections.{}: 'peer_id' does not exist", connection_id.b)));
                }
                if c.network_w_digest.network.connections.contains_key(connection_id) {
                    return Err(HttpResponse::Forbidden().body(format!("connection '{}' already exists", connection_id)));
                }
                if connection_id.a == connection_id.b {
                    return Err(HttpResponse::Forbidden().body(format!("loopback connection detected: {}", connection_id)));
                }

                // If deserialization succeeds, pre_shared_key is already validated.
                // If deserialization succeeds, allowed_ips_a_to_b is already validated.
                // If deserialization succeeds, allowed_ips_b_to_a is already validated.
                validate_conn_persistent_keepalive(&connection_details.persistent_keepalive).map_err(|e| {
                    HttpResponse::BadRequest().body(format!("added_connections.{}.persistent_keepalive: {}", connection_id, e))
                })?;

                c.network_w_digest
                    .network
                    .connections
                    .insert(connection_id.clone(), connection_details.clone());
                changed_config = true;
            }
        }
    }

    // process removed_connections
    if let Some(removed_connections) = &change_sum.removed_connections {
        for connection_id in removed_connections {
            {
                c.network_w_digest.network.connections.remove(connection_id);
                changed_config = true;
            }
        }
    }
    if !changed_config {
        log::debug!("nothing to update");
        return Err(HttpResponse::BadRequest().body("nothing to update"));
    }
    post_mg_config_w_digest!(c);
    log::info!("config updated");

    if c.agent.vpn.enabled {
        sync_conf(&c.clone().to_config()).map_err(|e| {
            log::error!("{e}");
            HttpResponse::InternalServerError().body("unable to synchronize config")
        })?;
    }

    Ok(HttpResponse::Ok().json(json!(change_sum)))
}

pub(crate) fn post_network_reserve_address() -> Result<HttpResponse, HttpResponse> {
    let mut c = get_mg_config_w_digest!();
    remove_expired_reservations(&mut c.network_w_digest.network);
    let next_address = network::get_next_available_address(&c.network_w_digest.network)
        .ok_or_else(|| HttpResponse::Conflict().body("No more IP addresses available in the pool".to_string()))?;

    let reservation_peer_id = Uuid::new_v4();
    let reservation_valid_until = Utc::now() + Duration::minutes(10);
    c.network_w_digest.network.reservations.insert(next_address, ReservationData {
        peer_id: reservation_peer_id,
        valid_until: reservation_valid_until,
    });
    post_mg_config_w_digest!(c);
    log::info!("reserved address {} for {} until {}", next_address, reservation_peer_id, reservation_valid_until);
    
    Ok(HttpResponse::Ok().json(json!({
        "address": next_address,
        "peer_id": reservation_peer_id,
        "valid_until": reservation_valid_until
    })))
}
