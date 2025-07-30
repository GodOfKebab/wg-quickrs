use crate::macros::*;
use crate::WIREGUARD_CONFIG_FILE;
use config_wasm::get_peer_wg_config;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};
use std::{fs, io};

pub(crate) fn disable_wireguard(config: &config_wasm::types::Config) -> Result<(), io::Error> {
    match Command::new("wg-quick")
        .arg("down")
        .arg(config.network.identifier.clone())
        .output() {
        Ok(output) => {
            log::info!("$ wg-quick down {}", config.network.identifier.clone());
            if !output.stdout.is_empty() { log::info!("{}", String::from_utf8_lossy(&output.stdout)); }
            if !output.stderr.is_empty() { log::warn!("{}", String::from_utf8_lossy(&output.stderr)); }
            if output.status.success() { return Ok(()); }
            Err(io::Error::from(io::ErrorKind::Other))
        }
        Err(e) => {
            Err(e)
        }
    }
}

pub(crate) fn enable_wireguard(config: &config_wasm::types::Config) -> Result<(), io::Error> {
    match Command::new("wg-quick")
        .arg("up")
        .arg(config.network.identifier.clone())
        .output() {
        Ok(output) => {
            log::info!("$ wg-quick up {}", config.network.identifier.clone());
            if !output.stdout.is_empty() { log::info!("{}", String::from_utf8_lossy(&output.stdout)); }
            if !output.stderr.is_empty() { log::warn!("{}", String::from_utf8_lossy(&output.stderr)); }
            if output.status.success() { return Ok(()); }
            Err(io::Error::from(io::ErrorKind::Other))
        }
        Err(e) => {
            Err(e)
        }
    }
}

pub(crate) fn update_wireguard_conf_file(config: &config_wasm::types::Config) -> Result<(), io::Error> {
    // generate .conf content
    let wg_conf = match get_peer_wg_config(&config.network, config.network.this_peer.clone(), full_version!()) {
        Ok(n) => n,
        Err(e) => {
            return Err(e);
        }
    };

    // write the content to the .conf file
    let config_path = WIREGUARD_CONFIG_FILE.get().unwrap();
    // make sure the parent directory exists
    match fs::create_dir_all(config_path.parent().unwrap()) {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    };
    // open the file with write-only permissions
    let mut file = match File::create(config_path) {
        Ok(f) => f,
        Err(e) => {
            return Err(e);
        }
    };
    // dump the new conf to the file
    match file.write_all(wg_conf.as_bytes()) {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    };
    return Ok(());
}

pub(crate) fn start_wireguard_tunnel(config: &config_wasm::types::Config) -> Result<(), io::Error> {
    // override .conf from .yml
    match update_wireguard_conf_file(config) {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    };
    match disable_wireguard(config) {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    };
    match enable_wireguard(config) {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    };
    log::info!("wireguard tunnel accessible at {}:{}", config.agent.address, config.agent.vpn.port);
    return Ok(());
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct PublicPrivateKey {
    public_key: String,
    private_key: String,
}
pub(crate) fn get_wireguard_public_private_keys() -> Result<PublicPrivateKey, io::Error> {
    let mut res = PublicPrivateKey {
        public_key: "".parse().unwrap(),
        private_key: "".parse().unwrap(),
    };

    match Command::new("wg")
        .arg("genkey")
        .output() {
        Ok(output) => {
            log::info!("$ wg genkey");
            if !output.stdout.is_empty() { log::info!("{}", String::from_utf8_lossy(&output.stdout)); }
            if !output.stderr.is_empty() { log::warn!("{}", String::from_utf8_lossy(&output.stderr)); }
            if !output.status.success() { return Err(io::Error::from(io::ErrorKind::Other)); }
            res.private_key = String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
        Err(e) => {
            return Err(e);
        }
    };

    match Command::new("wg")
        .arg("pubkey")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn() {
        Ok(mut child) => {
            if let Some(stdin) = child.stdin.as_mut() {
                match stdin.write_all(res.private_key.as_bytes()) {
                    Ok(_) => {}
                    Err(e) => { return Err(e); }
                }
            } else { return Err(io::Error::from(io::ErrorKind::Other)); }
            match child.wait_with_output() {
                Ok(output) => {
                    log::info!("$ wg genkey");
                    if !output.stdout.is_empty() { log::info!("{}", String::from_utf8_lossy(&output.stdout)); }
                    if !output.stderr.is_empty() { log::warn!("{}", String::from_utf8_lossy(&output.stderr)); }
                    if !output.status.success() { return Err(io::Error::from(io::ErrorKind::Other)); }
                    res.public_key = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    return Ok(res);
                }
                Err(e) => { return Err(e); }
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub(crate) fn get_wireguard_pre_shared_key() -> Result<String, io::Error> {
    match Command::new("wg")
        .arg("genpsk")
        .output() {
        Ok(output) => {
            log::info!("$ wg genpsk");
            if !output.stdout.is_empty() { log::info!("{}", String::from_utf8_lossy(&output.stdout)); }
            if !output.stderr.is_empty() { log::warn!("{}", String::from_utf8_lossy(&output.stderr)); }
            if output.status.success() { return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string()); }
            Err(io::Error::from(io::ErrorKind::Other))
        }
        Err(e) => {
            Err(e)
        }
    }
}
