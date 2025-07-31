use std::io;

pub mod types;

pub fn get_peer_wg_config(
    network: &types::Network,
    peer_id: String,
    version: &str,
) -> Result<String, io::Error> {
    let this_peer = match network.peers.get(&peer_id) {
        Some(n) => n,
        None => {
            return Err(io::Error::new(io::ErrorKind::NotFound, "peer not found"));
        }
    };

    let mut wg_conf = String::new();
    use std::fmt::Write as FmtWrite; // brings `write!` macro for String

    writeln!(wg_conf, "# auto-generated using wg-rusteze ({version})").unwrap();
    writeln!(
        wg_conf,
        "# wg-rusteze network identifier: {}\n",
        network.identifier
    )
    .unwrap();

    // Peer fields
    writeln!(wg_conf, "# Peer: {} ({})", this_peer.name, peer_id).unwrap();
    writeln!(wg_conf, "[Interface]").unwrap();
    writeln!(wg_conf, "PrivateKey = {}", this_peer.private_key).unwrap();
    writeln!(wg_conf, "Address = {}/24", this_peer.address).unwrap();

    if this_peer.endpoint.enabled {
        if let Some((_host, port)) = this_peer.endpoint.value.rsplit_once(':') {
            writeln!(wg_conf, "ListenPort = {port}").unwrap();
        }
    }
    if this_peer.dns.enabled {
        writeln!(wg_conf, "DNS = {}", this_peer.dns.value).unwrap();
    }
    if this_peer.mtu.enabled {
        writeln!(wg_conf, "MTU = {}", this_peer.mtu.value).unwrap();
    }
    let script_fields = &this_peer.scripts;
    if script_fields.pre_up.enabled {
        writeln!(wg_conf, "PreUp = {}", script_fields.pre_up.value).unwrap();
    }
    if script_fields.post_up.enabled {
        writeln!(wg_conf, "PostUp = {}", script_fields.post_up.value).unwrap();
    }
    if script_fields.pre_down.enabled {
        writeln!(wg_conf, "PreDown = {}", script_fields.pre_down.value).unwrap();
    }
    if script_fields.post_down.enabled {
        writeln!(wg_conf, "PostDown = {}", script_fields.post_down.value).unwrap();
    }
    writeln!(wg_conf).unwrap();

    // connection fields
    for (connection_id, connection_details) in network.connections.clone().into_iter() {
        if !connection_id.contains(&peer_id) {
            continue;
        }
        if !connection_details.enabled {
            continue;
        }

        let parts: Vec<&str> = connection_id.split('*').collect();
        if parts.len() != 2 {
            continue;
        } // or handle error
        let (other_peer_id, allowed_ips) = if parts[0] == peer_id {
            (parts[1], &connection_details.allowed_ips_a_to_b)
        } else {
            (parts[0], &connection_details.allowed_ips_b_to_a)
        };
        let other_peer_details = match network.peers.get(other_peer_id) {
            Some(n) => n,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "other peer not found",
                ));
            }
        };
        writeln!(
            wg_conf,
            "# Linked Peer: {} ({})",
            other_peer_details.name, other_peer_id
        )
        .unwrap();
        writeln!(wg_conf, "[Peer]").unwrap();
        writeln!(wg_conf, "PublicKey = {}", other_peer_details.public_key).unwrap();
        writeln!(
            wg_conf,
            "PresharedKey = {}",
            connection_details.pre_shared_key
        )
        .unwrap();
        writeln!(wg_conf, "AllowedIPs = {allowed_ips}").unwrap();

        if connection_details.persistent_keepalive.enabled {
            writeln!(
                wg_conf,
                "PersistentKeepalive = {}",
                connection_details.persistent_keepalive.value
            )
            .unwrap();
        }
        if other_peer_details.endpoint.enabled {
            writeln!(wg_conf, "Endpoint = {}", other_peer_details.endpoint.value).unwrap();
        }
        writeln!(wg_conf).unwrap();
    }
    Ok(wg_conf)
}

pub fn get_connection_id(peer1: &str, peer2: &str) -> String {
    if peer1 > peer2 {
        format!("{peer1}*{peer2}")
    } else {
        format!("{peer2}*{peer1}")
    }
}

#[cfg(target_arch = "wasm32")]
use serde_wasm_bindgen;
// Only include this when compiling to wasm32
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_peer_wg_config_frontend(network_js: JsValue, peer_id: String, version: &str) -> String {
    let network: types::Network = serde_wasm_bindgen::from_value(network_js).unwrap();
    get_peer_wg_config(&network, peer_id, version).unwrap()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_connection_id_frontend(peer1: &str, peer2: &str) -> String {
    get_connection_id(peer1, peer2)
}
