use ipnet::Ipv4Net;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddrV4};

pub mod types;

pub fn get_peer_wg_config(
    network: &types::Network,
    peer_id: String,
    version: &str,
) -> Result<String, WireGuardLibError> {
    let this_peer = match network.peers.get(&peer_id) {
        Some(n) => n,
        None => {
            return Err(WireGuardLibError::PeerNotFound(format!(
                "peer_id: {peer_id}"
            )));
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
                return Err(WireGuardLibError::PeerNotFound(format!(
                    "peer_id: {peer_id}"
                )));
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

#[derive(Debug, Serialize)]
struct CheckResult {
    status: bool,
    msg: String,
}

#[derive(Debug, Clone, Deserialize)]
struct FieldValue {
    str: String,
    enabled_value: types::EnabledValue,
}

// Helper: plain IPv4
fn is_ipv4(s: &str) -> bool {
    s.parse::<Ipv4Addr>().is_ok()
}

// Helper: IPv4 + port
fn is_ipv4_with_port(s: &str) -> bool {
    s.parse::<SocketAddrV4>().is_ok()
}

// Helper: CIDR IPv4 network
fn is_cidr(s: &str) -> bool {
    s.parse::<Ipv4Net>().is_ok()
}

// Helper: FQDN + port
fn is_fqdn_with_port(s: &str) -> bool {
    // Split on the last colon to separate the hostname and port
    match s.rsplit_once(':') {
        Some((hostname, port_str)) => {
            // port 0-65535 is valid: comparison is useless due to type limits
            if port_str.parse::<u16>().is_err() {
                return false;
            }
            // Validate hostname with validate-hostname crate
            hostname_validator::is_valid(hostname)
        }
        None => false, // no colon, no port
    }
}

fn check_field(field_name: &str, field_variable: &FieldValue) -> CheckResult {
    let mut ret = CheckResult {
        status: false,
        msg: String::new(),
    };

    println!(
        "Checking field: {field_name} with value: {field_variable:?}"
    );

    match field_name {
        // UUID v4 check
        "peerId" => {
            let re_uuid = Regex::new(
                r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$",
            )
            .unwrap();
            ret.status = re_uuid.is_match(&field_variable.str);
            if !ret.status {
                ret.msg = "peerId needs to follow uuid4 standards".into();
            }
        }

        "name" => {
            ret.status = !field_variable.str.is_empty();
            if !ret.status {
                ret.msg = "name cannot be empty".into();
            }
        }

        // TODO: check subnet
        // TODO: check to see if a duplicate exists
        "address" => {
            ret.status = is_ipv4(&field_variable.str);
            if !ret.status {
                ret.msg = "address is not IPv4".into();
            }
        }

        "endpoint" => {
            if field_variable.enabled_value.enabled
                && !(is_ipv4_with_port(&field_variable.enabled_value.value)
                    || is_fqdn_with_port(&field_variable.enabled_value.value))
            {
                ret.status = false;
                ret.msg = "endpoint is not IPv4 nor an FQDN".into();
            } else {
                ret.status = true;
            }
        }

        "dns" => {
            ret.status = true;
            if field_variable.enabled_value.enabled {
                // Allow multiple DNS servers, comma-separated
                ret.status = field_variable
                    .enabled_value
                    .value
                    .split(',')
                    .all(|addr| is_ipv4(addr.trim()));
            }
            if !ret.status {
                ret.msg = "DNS is invalid".into();
            }
        }

        "mtu" => {
            ret.status = true;
            if let Ok(v) = field_variable.enabled_value.value.parse::<i32>() {
                ret.status = v > 0 && v < 65536;
            } else {
                ret.status = false; // not a number
            }
            if !ret.status {
                ret.msg = "MTU is invalid".into();
            }
        }

        "script" => {
            ret.status = true;
            if field_variable.enabled_value.enabled {
                let re = Regex::new(r"^.*;\s*$").unwrap();
                if !re.is_match(&field_variable.enabled_value.value) {
                    ret.status = false;
                }
            }
            if !ret.status {
                ret.msg = "script needs to end with a semicolon".into();
            }
        }

        "allowed_ips_a_to_b" | "allowed_ips_b_to_a" => {
            ret.status = field_variable
                .str
                .split(',')
                .all(|cidr| is_cidr(cidr.trim()));
            if !ret.status {
                ret.msg = "AllowedIPs is not in CIDR format".into();
            }
        }

        "persistent_keepalive" => {
            ret.status = true;
            if let Ok(v) = field_variable.enabled_value.value.parse::<i32>() {
                ret.status = v > 0 && v < 65536;
            } else {
                ret.status = false; // not a number
            }
            if !ret.status {
                ret.msg = "Persistent Keepalive is invalid".into();
            }
        }

        _ => {
            ret.status = false;
            ret.msg = "field doesn't exist".into();
        }
    }

    ret
}
use crate::types::WireGuardLibError;
// Only include these when compiling to wasm32
#[cfg(target_arch = "wasm32")]
use serde_wasm_bindgen;
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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn check_field_frontend(field_name: &str, field_variable_json: &str) -> String {
    println!(
        "Checking field: {} with value: {}",
        field_name, field_variable_json
    );
    match serde_json::from_str::<FieldValue>(field_variable_json) {
        Ok(field_variable) => {
            let ret = check_field(field_name, &field_variable);
            serde_json::to_string(&ret)
                .unwrap_or_else(|_| r#"{"status":false,"msg":"Failed to serialize result"}"#.into())
        }
        Err(_) => r#"{"status":false,"msg":"Invalid JSON input"}"#.into(),
    }
}
