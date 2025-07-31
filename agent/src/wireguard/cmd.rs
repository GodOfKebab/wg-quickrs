use crate::WIREGUARD_CONFIG_FILE;
use crate::macros::*;
use config_wasm::get_peer_wg_config;
use config_wasm::types::{Config, TelemetryDatum, WireGuardStatus};
use once_cell::sync::Lazy;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::{fs, io};
use tempfile::NamedTempFile;

static WG_INTERFACE: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("".to_string()));

pub(crate) fn status_tunnel() -> Result<WireGuardStatus, io::Error> {
    let wg_interface_mut = match WG_INTERFACE.lock() {
        Ok(lock) => lock,
        Err(_e) => {
            return Err(io::Error::from(io::ErrorKind::Other));
        }
    };
    if (*wg_interface_mut).is_empty() {
        return Ok(WireGuardStatus::DOWN);
    }
    Ok(WireGuardStatus::UP)
}

pub(crate) fn show_dump(config: &Config) -> Result<HashMap<String, TelemetryDatum>, io::Error> {
    let wg_interface_mut = match WG_INTERFACE.lock() {
        Ok(lock) => lock,
        Err(_e) => {
            return Err(io::Error::from(io::ErrorKind::Other));
        }
    };
    if (*wg_interface_mut).is_empty() {
        return Err(io::Error::from(io::ErrorKind::Other));
    }

    // sudo wg show INTERFACE dump
    match Command::new("sudo")
        .arg("wg")
        .arg("show")
        .arg(&*wg_interface_mut)
        .arg("dump")
        .output()
    {
        Ok(output) => {
            log::info!("$ sudo wg show {} dump", &*wg_interface_mut);
            if !output.stdout.is_empty() {
                log::debug!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                log::warn!("{}", String::from_utf8_lossy(&output.stderr));
            }
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
                        if peer_details.public_key != public_key {
                            continue;
                        }
                        let transfer_rx = parts[5].parse::<u64>().unwrap_or(0);
                        let transfer_tx = parts[6].parse::<u64>().unwrap_or(0);
                        let connection_id =
                            config_wasm::get_connection_id(&config.network.this_peer, &peer_id);

                        telemetry.insert(
                            connection_id.clone(),
                            TelemetryDatum {
                                latest_handshake_at: parts[4].parse::<u64>().unwrap_or(0),
                                transfer_a_to_b: if connection_id
                                    .starts_with(&config.network.this_peer)
                                {
                                    transfer_tx
                                } else {
                                    transfer_rx
                                },
                                transfer_b_to_a: if connection_id
                                    .starts_with(&config.network.this_peer)
                                {
                                    transfer_rx
                                } else {
                                    transfer_tx
                                },
                            },
                        );
                        break;
                    }
                }

                return Ok(telemetry);
            }
            Err(io::Error::from(io::ErrorKind::Other))
        }
        Err(e) => Err(e),
    }
}

pub(crate) fn sync_conf(config: &Config) -> Result<(), io::Error> {
    match update_conf_file(config) {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    };

    // wg-quick strip WG_INTERFACE
    let stripped_output = match Command::new("wg-quick")
        .arg("strip")
        .arg(config.network.identifier.clone())
        .output()
    {
        Ok(output) => {
            log::info!("$ wg-quick strip {}", config.network.identifier.clone());
            if !output.stdout.is_empty() {
                log::debug!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                log::warn!("{}", String::from_utf8_lossy(&output.stderr));
            }
            if !output.status.success() {
                return Err(io::Error::from(io::ErrorKind::Other));
            }
            output
        }
        Err(e) => {
            return Err(e);
        }
    };

    // Write to a temp file
    let mut temp = match NamedTempFile::new() {
        Ok(file) => file,
        Err(e) => {
            return Err(e);
        }
    };
    match temp.write_all(&stripped_output.stdout) {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    };
    let temp_path = temp.path().to_owned(); // Save path before drop

    let wg_interface_mut = match WG_INTERFACE.lock() {
        Ok(lock) => lock,
        Err(_e) => {
            return Err(io::Error::from(io::ErrorKind::Other));
        }
    };

    // wg syncconf WG_INTERFACE <(wg-quick strip WG_INTERFACE)
    match Command::new("sudo")
        .arg("wg")
        .arg("syncconf")
        .arg(&*wg_interface_mut)
        .arg(temp_path)
        .output()
    {
        Ok(output) => {
            log::info!(
                "$ wg syncconf {} <(wg-quick strip {})",
                &*wg_interface_mut,
                config.network.identifier.clone()
            );
            if !output.stdout.is_empty() {
                log::debug!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                log::warn!("{}", String::from_utf8_lossy(&output.stderr));
            }
            if !output.status.success() {
                return Err(io::Error::from(io::ErrorKind::Other));
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub(crate) fn disable_tunnel(config: &Config) -> Result<(), io::Error> {
    // sudo wg-quick down INTERFACE
    match Command::new("sudo")
        .arg("wg-quick")
        .arg("down")
        .arg(config.network.identifier.clone())
        .output()
    {
        Ok(output) => {
            log::info!("$ sudo wg-quick down {}", config.network.identifier.clone());
            if !output.stdout.is_empty() {
                log::debug!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                log::warn!("{}", String::from_utf8_lossy(&output.stderr));
            }
            if output.status.success() {
                match WG_INTERFACE.lock() {
                    Ok(mut wg_interface_mut) => {
                        *wg_interface_mut = "".to_string();
                    }
                    Err(_e) => {
                        return Err(io::Error::from(io::ErrorKind::Other));
                    }
                };
                return Ok(());
            }
            Err(io::Error::from(io::ErrorKind::Other))
        }
        Err(e) => Err(e),
    }
}

pub(crate) fn enable_tunnel(config: &Config) -> Result<(), io::Error> {
    // sudo wg-quick up INTERFACE
    match Command::new("sudo")
        .arg("wg-quick")
        .arg("up")
        .arg(config.network.identifier.clone())
        .output()
    {
        Ok(output) => {
            log::info!("$ sudo wg-quick up {}", config.network.identifier.clone());
            if !output.stdout.is_empty() {
                log::debug!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                log::warn!("{}", String::from_utf8_lossy(&output.stderr));
            }
            if !output.stderr.is_empty() {
                let mut wg_interface_mut = match WG_INTERFACE.lock() {
                    Ok(lock) => lock,
                    Err(_e) => {
                        return Err(io::Error::from(io::ErrorKind::Other));
                    }
                };
                match String::from_utf8_lossy(&output.stderr)
                    .lines()
                    .find(|line| line.contains("[+] Interface for wg-rusteze-home is"))
                    .map(|line| line.to_string())
                {
                    Some(line) => match line.split_whitespace().last() {
                        Some(word) => {
                            *wg_interface_mut = word.to_string();
                        }
                        None => {
                            return Err(io::Error::from(io::ErrorKind::Other));
                        }
                    },
                    None => {
                        *wg_interface_mut = config.network.identifier.clone();
                    }
                }
            }
            if output.status.success() {
                return Ok(());
            }
            Err(io::Error::from(io::ErrorKind::Other))
        }
        Err(e) => Err(e),
    }
}

pub(crate) fn update_conf_file(config: &Config) -> Result<(), io::Error> {
    // generate .conf content
    let wg_conf = match get_peer_wg_config(
        &config.network,
        config.network.this_peer.clone(),
        full_version!(),
    ) {
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
    Ok(())
}

pub(crate) fn start_tunnel(config: &Config) -> Result<(), io::Error> {
    // override .conf from .yml
    update_conf_file(config)?;

    // disable then enable and ignore any error when disabling
    let _ = disable_tunnel(config);
    enable_tunnel(config)?;

    log::info!(
        "wireguard tunnel accessible at {}:{}",
        config.agent.address,
        config.agent.vpn.port
    );
    Ok(())
}

pub(crate) fn get_public_private_keys() -> Result<Value, io::Error> {
    #[allow(unused_assignments)]
    let mut private_key = "".parse().unwrap();

    // wg genkey
    match Command::new("wg").arg("genkey").output() {
        Ok(output) => {
            log::info!("$ wg genkey");
            if !output.stdout.is_empty() {
                log::debug!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                log::warn!("{}", String::from_utf8_lossy(&output.stderr));
            }
            if !output.status.success() {
                return Err(io::Error::from(io::ErrorKind::Other));
            }
            private_key = String::from_utf8_lossy(&output.stdout).trim().to_string();
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
        .spawn()
    {
        Ok(mut child) => {
            if let Some(stdin) = child.stdin.as_mut() {
                match stdin.write_all(private_key.as_bytes()) {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(e);
                    }
                }
            } else {
                return Err(io::Error::from(io::ErrorKind::Other));
            }
            match child.wait_with_output() {
                Ok(output) => {
                    log::info!("$ wg genkey");
                    if !output.stdout.is_empty() {
                        log::debug!("{}", String::from_utf8_lossy(&output.stdout));
                    }
                    if !output.stderr.is_empty() {
                        log::warn!("{}", String::from_utf8_lossy(&output.stderr));
                    }
                    if !output.status.success() {
                        return Err(io::Error::from(io::ErrorKind::Other));
                    }
                    Ok(json!({
                        "private_key": private_key,
                        "public_key": String::from_utf8_lossy(&output.stdout).trim().to_string(),
                    }))
                }
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

pub(crate) fn get_pre_shared_key() -> Result<Value, io::Error> {
    // wg genpsk
    match Command::new("wg").arg("genpsk").output() {
        Ok(output) => {
            log::info!("$ wg genpsk");
            if !output.stdout.is_empty() {
                log::debug!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                log::warn!("{}", String::from_utf8_lossy(&output.stderr));
            }
            if output.status.success() {
                return Ok(json!({
                    "pre_shared_key": String::from_utf8_lossy(&output.stdout).trim().to_string(),
                }));
            }
            Err(io::Error::from(io::ErrorKind::Other))
        }
        Err(e) => Err(e),
    }
}
