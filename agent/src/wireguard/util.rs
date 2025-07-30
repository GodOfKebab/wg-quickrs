use crate::macros::*;
use crate::WIREGUARD_CONFIG_FILE;
use actix_web::{HttpResponse, Responder};
use config_wasm::get_peer_wg_config;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};

pub(crate) fn start_wireguard_tunnel(config: &config_wasm::types::Config) {
    let wg_conf = match get_peer_wg_config(&config.network, config.network.this_peer.clone(), full_version!()) {
        Ok(n) => n,
        Err(e) => {
            log::error!("{}", e);
            return;
        },
    };

    let config_path = WIREGUARD_CONFIG_FILE
        .get()
        .expect("WIREGUARD_CONFIG_FILE not set");

    let mut file = File::create(config_path).expect("Failed to open config file");
    file.write_all(wg_conf.as_bytes()).expect("Failed to write to config file");

    // log::info!("wireguard tunnel accessible at {}:{}", config.agent.address, config.agent.vpn.port);
    // log::info!("wireguard tunnel address: {} subnet: {}", this_peer.address, config.network.subnet);
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

