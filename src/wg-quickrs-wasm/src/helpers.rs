use crate::types;
use crate::types::WireGuardLibError;
use base64::{engine::general_purpose, Engine as _};
use x25519_dalek::{PublicKey, StaticSecret};
use rand::RngCore;


pub fn get_peer_wg_config(
    network: &types::Network,
    peer_id: String,
    version: &str,
    peer_hidden_scripts: Option<String>,
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

    writeln!(wg_conf, "# auto-generated using wg-quickrs ({version})").unwrap();
    writeln!(
        wg_conf,
        "# wg-quickrs network identifier: {}\n",
        network.identifier
    )
    .unwrap();

    // Peer fields
    writeln!(wg_conf, "# Peer: {} ({})", this_peer.name, peer_id).unwrap();
    writeln!(wg_conf, "[Interface]").unwrap();
    writeln!(wg_conf, "PrivateKey = {}", this_peer.private_key).unwrap();
    writeln!(wg_conf, "Address = {}/24", this_peer.address).unwrap();

    if this_peer.endpoint.enabled
        && let Some((_host, port)) = this_peer.endpoint.value.rsplit_once(':')
    {
        writeln!(wg_conf, "ListenPort = {port}").unwrap();
    }
    if this_peer.dns.enabled {
        writeln!(wg_conf, "DNS = {}", this_peer.dns.value).unwrap();
    }
    if this_peer.mtu.enabled {
        writeln!(wg_conf, "MTU = {}", this_peer.mtu.value).unwrap();
    }
    let script_fields = &this_peer.scripts;
    if let Some(hidden_scripts) = peer_hidden_scripts {
        writeln!(wg_conf, "{}", hidden_scripts).unwrap();
    }
    for script_field in &script_fields.pre_up {
        if script_field.enabled {
            writeln!(wg_conf, "PreUp = {}", script_field.value).unwrap();
        }
    }
    for script_field in &script_fields.post_up {
        if script_field.enabled {
            writeln!(wg_conf, "PostUp = {}", script_field.value).unwrap();
        }
    }
    for script_field in &script_fields.pre_down {
        if script_field.enabled {
            writeln!(wg_conf, "PreDown = {}", script_field.value).unwrap();
        }
    }
    for script_field in &script_fields.post_down {
        if script_field.enabled {
            writeln!(wg_conf, "PostDown = {}", script_field.value).unwrap();
        }
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
        writeln!(wg_conf, "PublicKey = {}", wg_public_key_from_private_key(&other_peer_details.private_key)?).unwrap();
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


/// Compute a WireGuard public key from a base64-encoded private key.
pub fn wg_public_key_from_private_key(base64_priv: &str) -> Result<String, WireGuardLibError> {
    // Decode base64-encoded private key (32 bytes)
    let priv_bytes: [u8; 32] = general_purpose::STANDARD
        .decode(base64_priv.trim())
        .map_err(|e| WireGuardLibError::KeyDecodeFailed(e.to_string()))?
        .as_slice()
        .try_into()
        .map_err(|_| WireGuardLibError::KeyDecodeFailed("key != 32 bytes".to_string()))?;

    let secret = StaticSecret::from(priv_bytes);
    let public = PublicKey::from(&secret);
    Ok(general_purpose::STANDARD.encode(public.as_bytes()))
}


/// Generate a new WireGuard private key (Base64-encoded)
pub fn wg_generate_key() -> String {
    let mut key_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut key_bytes);
    general_purpose::STANDARD.encode(key_bytes)
}