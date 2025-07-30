use crate::conf;
use crate::macros::*;
use crate::WIREGUARD_CONFIG_FILE;
use actix_web::{HttpResponse, Responder};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::process::{Command, Stdio};


pub(crate) fn start_wireguard_tunnel(config: &conf::types::Config) {
    let this_peer = match config.network.peers.get(&config.network.this_peer) {
        Some(n) => n,
        None => {
            log::error!("unable to parse the wg-rusteze config file! unable to start wireguard tunnel");
            return
        },
    };

    let mut wg_conf = String::new();
    use std::fmt::Write as FmtWrite; // brings `write!` macro for String

    writeln!(wg_conf, "# auto-generated using wg-rusteze ({})", full_version!()).unwrap();
    writeln!(wg_conf, "# wg-rusteze network identifier: {}\n", config.network.identifier).unwrap();

    // Peer fields
    writeln!(wg_conf, "# Peer: {} ({})", this_peer.name, config.network.this_peer).unwrap();
    writeln!(wg_conf, "[Interface]").unwrap();
    writeln!(wg_conf, "PrivateKey = {}", this_peer.private_key).unwrap();
    writeln!(wg_conf, "Address = {}/24", this_peer.address).unwrap();

    if this_peer.endpoint.enabled {
        if let Some((_host, port)) = this_peer.endpoint.value.rsplit_once(':') {
            writeln!(wg_conf, "ListenPort = {}", port).unwrap();
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
    for (connection_id, connection_details) in config.network.connections.clone().into_iter() {
        if (!connection_id.contains(&config.network.this_peer)) { continue; }
        if (!connection_details.enabled) { continue; }

        let parts: Vec<&str> = connection_id.split('*').collect();
        if parts.len() != 2 { continue; } // or handle error
        let (other_peer_id, allowed_ips) = if parts[0] == config.network.this_peer {
            (parts[1], &connection_details.allowed_ips_a_to_b)
        } else {
            (parts[0], &connection_details.allowed_ips_b_to_a)
        };
        let other_peer_details = match config.network.peers.get(other_peer_id) {
            Some(n) => n,
            None => {
                log::error!("unable to parse the wg-rusteze config file! unable to start wireguard tunnel");
                return
            },
        };
        writeln!(wg_conf, "# Linked Peer: {} ({})", other_peer_details.name, other_peer_id).unwrap();
        writeln!(wg_conf, "[Peer]").unwrap();
        writeln!(wg_conf, "PublicKey = {}", other_peer_details.public_key).unwrap();
        writeln!(wg_conf, "PresharedKey = {}", connection_details.pre_shared_key).unwrap();
        writeln!(wg_conf, "AllowedIPs = {}", allowed_ips).unwrap();

        if connection_details.persistent_keepalive.enabled {
            writeln!(wg_conf, "PersistentKeepalive = {}", connection_details.persistent_keepalive.value).unwrap();
        }
        if other_peer_details.endpoint.enabled {
            writeln!(wg_conf, "Endpoint = {}", other_peer_details.endpoint.value).unwrap();
        }
        writeln!(wg_conf).unwrap();
    }

    let config_path = WIREGUARD_CONFIG_FILE
        .get()
        .expect("WIREGUARD_CONFIG_FILE not set");

    let mut file = File::create(config_path).expect("Failed to open config file");
    file.write_all(wg_conf.as_bytes()).expect("Failed to write to config file");

    log::info!("wireguard tunnel accessible at {}:{}", config.agent.address, config.agent.vpn.port);
    log::info!("wireguard tunnel address: {} subnet: {}", this_peer.address, config.network.subnet);
}


pub(crate) fn respond_get_wireguard_public_private_key() -> impl Responder {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct PublicPrivateKey {
        public_key: String,
        private_key: String,
    }

    // execute $ wg genkey
    let priv_key_resp = Command::new("wg")
        .arg("genkey")
        .output()
        .unwrap();
    if !priv_key_resp.status.success() {
        HttpResponse::InternalServerError();
    }
    let priv_key = String::from_utf8_lossy(&priv_key_resp.stdout);

    // execute $ echo ~PRIVATE_KEY~ | wg pubkey
    let mut pub_key_child = Command::new("wg")
        .arg("pubkey")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(stdin) = pub_key_child.stdin.as_mut() {
        stdin.write_all(priv_key.as_bytes()).unwrap();
    }
    let pub_key_resp = pub_key_child.wait_with_output().unwrap();
    if !pub_key_resp.status.success() {
        HttpResponse::InternalServerError();
    }
    let pub_key = String::from_utf8_lossy(&pub_key_resp.stdout);

    let body = PublicPrivateKey {
        public_key: pub_key.trim().to_string(),
        private_key: priv_key.trim().to_string(),
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&body).unwrap())
}

pub(crate) fn respond_get_wireguard_pre_shared_key() -> impl Responder {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct PreSharedKey {
        pre_shared_key: String,
    }

    // execute $ wg genpsk
    let pre_shared_key_resp = Command::new("wg")
        .arg("genpsk")
        .output()
        .unwrap();
    if !pre_shared_key_resp.status.success() {
        HttpResponse::InternalServerError();
    }
    let pre_shared_key = String::from_utf8_lossy(&pre_shared_key_resp.stdout);

    let body = PreSharedKey {
        pre_shared_key: pre_shared_key.trim().to_string(),
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&body).unwrap())
}

