use crate::macros::*;
use crate::WIREGUARD_CONFIG_FILE;
use config_wasm::get_peer_wg_config;
use config_wasm::types::{TelemetryDatum, WireGuardStatus};
use once_cell::sync::Lazy;
use serde::__private::ser::constrain;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::{fs, io};

static WG_INTERFACE: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("".to_string()));

pub(crate) fn status_wireguard() -> Result<WireGuardStatus, io::Error> {
    let wg_interface_mut = match WG_INTERFACE.lock() {
        Ok(lock) => { lock },
        Err(_e) => { return Err(io::Error::from(io::ErrorKind::Other)); }
    };
    if *wg_interface_mut == "" { return Ok(WireGuardStatus::DOWN); }
    Ok(WireGuardStatus::UP)
}

pub(crate) fn show_dump_wireguard(config: &config_wasm::types::Config) -> Result<HashMap<String, TelemetryDatum>, io::Error> {
    let wg_interface_mut = match WG_INTERFACE.lock() {
        Ok(lock) => { lock },
        Err(_e) => { return Err(io::Error::from(io::ErrorKind::Other)); }
    };
    if *wg_interface_mut == "" { return Err(io::Error::from(io::ErrorKind::Other)); }

    // sudo wg show INTERFACE dump
    match Command::new("sudo")
        .arg("wg")
        .arg("show")
        .arg(&*wg_interface_mut)
        .arg("dump")
        .output() {
        Ok(output) => {
            log::info!("$ sudo wg show {} dump", &*wg_interface_mut);
            if !output.stdout.is_empty() { log::info!("{}", String::from_utf8_lossy(&output.stdout)); }
            if !output.stderr.is_empty() { log::warn!("{}", String::from_utf8_lossy(&output.stderr)); }
            if output.status.success() {
                let mut telemetry = HashMap::<String, TelemetryDatum>::new();

                let dump = String::from_utf8_lossy(&output.stdout);
                for line in dump.trim().lines().skip(1) {
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() < 8 {
                        continue;
                    }
                    let public_key = parts[0];

                    for (peer_id, peer_details) in config.network.peers.clone() {
                        if peer_details.public_key != public_key { continue; }
                        let transfer_rx = parts[5].parse::<u64>().unwrap_or(0);
                        let transfer_tx = parts[6].parse::<u64>().unwrap_or(0);
                        let connection_id = config_wasm::get_connection_id(&config.network.this_peer, &peer_id);

                        telemetry.insert(connection_id.clone(), TelemetryDatum {
                            latest_handshake_at: parts[4].parse::<u64>().unwrap_or(0),
                            transfer_a_to_b: if connection_id.starts_with(&config.network.this_peer) { transfer_tx } else { transfer_rx },
                            transfer_b_to_a: if connection_id.starts_with(&config.network.this_peer) { transfer_rx } else { transfer_tx },
                        });
                        break;
                    }
                }

                return Ok(telemetry);
            }
            return Err(io::Error::from(io::ErrorKind::Other));
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub(crate) fn disable_wireguard(config: &config_wasm::types::Config) -> Result<(), io::Error> {
    // sudo wg-quick down INTERFACE
    match Command::new("sudo")
        .arg("wg-quick")
        .arg("down")
        .arg(config.network.identifier.clone())
        .output() {
        Ok(output) => {
            log::info!("$ sudo wg-quick down {}", config.network.identifier.clone());
            if !output.stdout.is_empty() { log::info!("{}", String::from_utf8_lossy(&output.stdout)); }
            if !output.stderr.is_empty() { log::warn!("{}", String::from_utf8_lossy(&output.stderr)); }
            if output.status.success() {
                match WG_INTERFACE.lock() {
                    Ok(mut wg_interface_mut) => { *wg_interface_mut = "".to_string(); },
                    Err(_e) => { return Err(io::Error::from(io::ErrorKind::Other)); }
                };
                return Ok(());
            }
            Err(io::Error::from(io::ErrorKind::Other))
        }
        Err(e) => {
            Err(e)
        }
    }
}

pub(crate) fn enable_wireguard(config: &config_wasm::types::Config) -> Result<(), io::Error> {
    // sudo wg-quick up INTERFACE
    match Command::new("sudo")
        .arg("wg-quick")
        .arg("up")
        .arg(config.network.identifier.clone())
        .output() {
        Ok(output) => {
            log::info!("$ sudo wg-quick up {}", config.network.identifier.clone());
            if !output.stdout.is_empty() { log::info!("{}", String::from_utf8_lossy(&output.stdout)); }
            if !output.stderr.is_empty() { log::warn!("{}", String::from_utf8_lossy(&output.stderr)); }
            if !output.stderr.is_empty() {
                let mut wg_interface_mut = match WG_INTERFACE.lock() {
                    Ok(lock) => lock,
                    Err(_e) => { return Err(io::Error::from(io::ErrorKind::Other)); }
                };
                match String::from_utf8_lossy(&output.stderr)
                    .lines()
                    .find(|line| line.contains("[+] Interface for wg-rusteze-home is"))
                    .map(|line| line.to_string()) {
                    Some(line) => {
                        match line.split_whitespace().last() {
                            Some(word) => { *wg_interface_mut = word.to_string(); },
                            None => { return Err(io::Error::from(io::ErrorKind::Other)); }
                        }
                    },
                    None => {
                        *wg_interface_mut = config.network.identifier.clone();
                    }
                }
            }
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
        Err(_e) => {}
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

    // wg genkey
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

    // wg genkey | wg pubkey
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
    // wg genpsk
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
