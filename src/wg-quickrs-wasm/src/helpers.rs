use crate::types::conf::{Network, EndpointAddress, WireGuardKey};
use crate::types::misc::{WireGuardLibError};
use x25519_dalek::{PublicKey, StaticSecret};
use rand::RngCore;
use uuid::Uuid;


pub fn get_peer_wg_config(
    network: &Network,
    peer_id: &Uuid,
    version: &str,
    stripped: bool,
) -> Result<String, WireGuardLibError> {
    let this_peer = match network.peers.get(peer_id) {
        Some(n) => n,
        None => {
            return Err(WireGuardLibError::PeerNotFound(*peer_id));
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
    if !stripped {
        writeln!(wg_conf, "Address = {}/24", this_peer.address).unwrap();
    }

    if this_peer.endpoint.enabled
    {
        writeln!(wg_conf, "ListenPort = {}", this_peer.endpoint.port).unwrap();
    }
    if !stripped {
        if this_peer.dns.enabled {
            writeln!(wg_conf, "DNS = {}", this_peer.dns.addresses.iter()
                .map(|net| net.to_string())
                .collect::<Vec<_>>()
                .join(", ")).unwrap();
        }
        if this_peer.mtu.enabled {
            writeln!(wg_conf, "MTU = {}", this_peer.mtu.value).unwrap();
        }
        let script_fields = &this_peer.scripts;
        for script_field in &script_fields.pre_up {
            if script_field.enabled {
                writeln!(wg_conf, "PreUp = {}", script_field.script).unwrap();
            }
        }
        for script_field in &script_fields.post_up {
            if script_field.enabled {
                writeln!(wg_conf, "PostUp = {}", script_field.script).unwrap();
            }
        }
        for script_field in &script_fields.pre_down {
            if script_field.enabled {
                writeln!(wg_conf, "PreDown = {}", script_field.script).unwrap();
            }
        }
        for script_field in &script_fields.post_down {
            if script_field.enabled {
                writeln!(wg_conf, "PostDown = {}", script_field.script).unwrap();
            }
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

        let (other_peer_id, allowed_ips) = if connection_id.a == *peer_id {
            (connection_id.b, &connection_details.allowed_ips_a_to_b)
        } else {
            (connection_id.a, &connection_details.allowed_ips_b_to_a)
        };
        let other_peer_details = match network.peers.get(&other_peer_id) {
            Some(n) => n,
            None => {
                return Err(WireGuardLibError::PeerNotFound(*peer_id));
            }
        };
        writeln!(wg_conf, "# Linked Peer: {} ({})", other_peer_details.name, other_peer_id).unwrap();
        writeln!(wg_conf, "[Peer]").unwrap();
        writeln!(wg_conf, "PublicKey = {}", wg_public_key_from_private_key(&other_peer_details.private_key)).unwrap();
        writeln!(wg_conf, "PresharedKey = {}", connection_details.pre_shared_key).unwrap();
        writeln!(wg_conf, "AllowedIPs = {}", allowed_ips.iter()
            .map(|net| net.to_string())
            .collect::<Vec<_>>()
            .join(", ")).unwrap();

        if connection_details.persistent_keepalive.enabled {
            writeln!(
                wg_conf,
                "PersistentKeepalive = {}",
                connection_details.persistent_keepalive.period
            )
            .unwrap();
        }
        if other_peer_details.endpoint.enabled {
            if let EndpointAddress::Ipv4(ipv4) = &other_peer_details.endpoint.address {
                writeln!(wg_conf, "Endpoint = {}:{}", ipv4.to_string(), other_peer_details.endpoint.port).unwrap();
            } else if let EndpointAddress::Hostname(hostname) = &other_peer_details.endpoint.address {
                writeln!(wg_conf, "Endpoint = {}:{}", hostname, other_peer_details.endpoint.port).unwrap();
            }
        }
        writeln!(wg_conf).unwrap();
    }
    Ok(wg_conf)
}

/// Compute a WireGuard public key with a private key.
pub fn wg_public_key_from_private_key(priv_bytes: &WireGuardKey) -> WireGuardKey {
    let secret = StaticSecret::from(*priv_bytes.as_bytes());
    let public = PublicKey::from(&secret);
    WireGuardKey(*public.as_bytes())
}


/// Generate a new WireGuard private key
pub fn wg_generate_key() -> WireGuardKey {
    let mut key_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut key_bytes);
    WireGuardKey(key_bytes)
}