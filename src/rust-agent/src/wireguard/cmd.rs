use crate::macros::*;
use crate::WIREGUARD_CONFIG_FILE;
use once_cell::sync::Lazy;
use rust_wasm::helpers::get_peer_wg_config;
use rust_wasm::types::{Config, TelemetryDatum, WireGuardStatus};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use tempfile::NamedTempFile;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WireGuardCommandError {
    #[error("wireguard::cmd::error::lock_failed -> failed to acquire lock: {0}")]
    LockFailed(String),
    #[error("wireguard::cmd::error::interface_missing -> wireguard interface doesn't exist")]
    InterfaceMissing,
    #[error("wireguard::cmd::error::command_exec_error -> command for {0} failed: {1}")]
    CommandExecError(String, std::io::Error),
    #[error(
        "wireguard::cmd::error::command_exec_not_successful -> command for {0} completed unsuccessfully"
    )]
    CommandExecNotSuccessful(String),
    #[error(
        "wireguard::cmd::error::folder_creation_error -> failed to create folder at {0} failed: {1}"
    )]
    FolderCreationError(PathBuf, std::io::Error),
    #[error(
        "wireguard::cmd::error::file_creation_error -> failed to create file at {0} failed: {1}"
    )]
    FileCreationError(PathBuf, std::io::Error),
    #[error("wireguard::cmd::error::file_write_error -> failed to write file at {0} failed: {1}")]
    FileWriteError(PathBuf, std::io::Error),
    #[error("wireguard::cmd::error::interface_sync_failed -> failed to sync wireguard interface")]
    InterfaceSyncFailed,
    #[error("wireguard::cmd::error::other -> unexpected error: {0}")]
    Other(String),
}

static WG_INTERFACE: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("".to_string()));

pub(crate) fn status_tunnel() -> Result<WireGuardStatus, WireGuardCommandError> {
    let wg_interface_mut = WG_INTERFACE
        .lock()
        .map_err(|e| WireGuardCommandError::LockFailed(e.to_string()))?;
    if (*wg_interface_mut).is_empty() {
        return Ok(WireGuardStatus::DOWN);
    }
    Ok(WireGuardStatus::UP)
}

pub(crate) fn show_dump(
    config: &Config,
) -> Result<HashMap<String, TelemetryDatum>, WireGuardCommandError> {
    let wg_interface_mut = WG_INTERFACE
        .lock()
        .map_err(|e| WireGuardCommandError::LockFailed(e.to_string()))?;

    if (*wg_interface_mut).is_empty() {
        return Err(WireGuardCommandError::InterfaceMissing);
    }

    // sudo wg show INTERFACE dump
    let readable_command = format!("$ sudo wg show {} dump", &*wg_interface_mut);
    match Command::new("sudo")
        .arg("wg")
        .arg("show")
        .arg(&*wg_interface_mut)
        .arg("dump")
        .output()
    {
        Ok(output) => {
            log::info!("{readable_command}");
            if !output.stdout.is_empty() {
                log::debug!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                log::warn!("{}", String::from_utf8_lossy(&output.stderr));
            }
            if !output.status.success() {
                return Err(WireGuardCommandError::CommandExecNotSuccessful(
                    readable_command,
                ));
            }
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
                        rust_wasm::helpers::get_connection_id(&config.network.this_peer, &peer_id);

                    telemetry.insert(
                        connection_id.clone(),
                        TelemetryDatum {
                            latest_handshake_at: parts[4].parse::<u64>().unwrap_or(0),
                            transfer_a_to_b: if connection_id.starts_with(&config.network.this_peer)
                            {
                                transfer_tx
                            } else {
                                transfer_rx
                            },
                            transfer_b_to_a: if connection_id.starts_with(&config.network.this_peer)
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
            Ok(telemetry)
        }
        Err(e) => Err(WireGuardCommandError::CommandExecError(readable_command, e)),
    }
}

pub(crate) fn sync_conf(config: &Config) -> Result<(), WireGuardCommandError> {
    match update_conf_file(config) {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    };

    // wg-quick strip WG_INTERFACE
    let readable_command = format!("$ wg-quick strip {}", config.network.identifier.clone());
    let stripped_output = match Command::new("wg-quick")
        .arg("strip")
        .arg(config.network.identifier.clone())
        .output()
    {
        Ok(output) => {
            log::info!("{readable_command}");
            if !output.stdout.is_empty() {
                log::debug!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                log::warn!("{}", String::from_utf8_lossy(&output.stderr));
            }
            if !output.status.success() {
                return Err(WireGuardCommandError::CommandExecNotSuccessful(
                    readable_command,
                ));
            }
            output
        }
        Err(e) => {
            return Err(WireGuardCommandError::CommandExecError(readable_command, e));
        }
    };

    // Write to a temp file
    let mut temp = match NamedTempFile::new() {
        Ok(file) => file,
        Err(e) => {
            return Err(WireGuardCommandError::Other(e.to_string()));
        }
    };
    match temp.write_all(&stripped_output.stdout) {
        Ok(_) => {}
        Err(e) => {
            return Err(WireGuardCommandError::FileWriteError(
                PathBuf::from(temp.path()),
                e,
            ));
        }
    };
    let temp_path = temp.path().to_owned(); // Save path before drop

    let wg_interface_mut = WG_INTERFACE
        .lock()
        .map_err(|e| WireGuardCommandError::LockFailed(e.to_string()))?;

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
                return Err(WireGuardCommandError::InterfaceSyncFailed);
            }
            Ok(())
        }
        Err(e) => Err(WireGuardCommandError::CommandExecError(readable_command, e)),
    }
}

pub(crate) fn disable_tunnel(config: &Config) -> Result<(), WireGuardCommandError> {
    // sudo wg-quick down INTERFACE
    let readable_command = format!("$ sudo wg-quick down {}", config.network.identifier.clone());
    match Command::new("sudo")
        .arg("wg-quick")
        .arg("down")
        .arg(config.network.identifier.clone())
        .output()
    {
        Ok(output) => {
            log::info!("{readable_command}");
            if !output.stdout.is_empty() {
                log::debug!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                log::warn!("{}", String::from_utf8_lossy(&output.stderr));
            }
            if !output.status.success() {
                return Err(WireGuardCommandError::CommandExecNotSuccessful(
                    readable_command,
                ));
            }
            *WG_INTERFACE
                .lock()
                .map_err(|e| WireGuardCommandError::LockFailed(e.to_string()))? = String::new();
            Ok(())
        }
        Err(e) => Err(WireGuardCommandError::CommandExecError(readable_command, e)),
    }
}

pub(crate) fn enable_tunnel(config: &Config) -> Result<(), WireGuardCommandError> {
    // sudo wg-quick up INTERFACE
    let readable_command = format!("$ sudo wg-quick up {}", config.network.identifier.clone());
    match Command::new("sudo")
        .arg("wg-quick")
        .arg("up")
        .arg(config.network.identifier.clone())
        .output()
    {
        Ok(output) => {
            log::info!("{readable_command}");
            if !output.stdout.is_empty() {
                log::debug!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                log::warn!("{}", String::from_utf8_lossy(&output.stderr));
            }
            if !output.stderr.is_empty() {
                let mut wg_interface_mut = WG_INTERFACE
                    .lock()
                    .map_err(|e| WireGuardCommandError::LockFailed(e.to_string()))?;

                match String::from_utf8_lossy(&output.stderr)
                    .lines()
                    .find(|line| {
                        line.contains(&format!(
                            "[+] Interface for {} is",
                            config.network.identifier.clone()
                        ))
                    })
                    .map(|line| line.to_string())
                {
                    Some(line) => match line.split_whitespace().last() {
                        Some(word) => {
                            *wg_interface_mut = word.to_string();
                        }
                        None => {
                            return Err(WireGuardCommandError::InterfaceMissing);
                        }
                    },
                    None => {
                        *wg_interface_mut = config.network.identifier.clone();
                    }
                }
            }
            if !output.status.success() {
                return Err(WireGuardCommandError::CommandExecNotSuccessful(
                    readable_command,
                ));
            }
            Ok(())
        }
        Err(e) => Err(WireGuardCommandError::CommandExecError(readable_command, e)),
    }
}

pub(crate) fn update_conf_file(config: &Config) -> Result<(), WireGuardCommandError> {
    // generate .conf content with hidden scripts
    let mut hidden_scripts = None;
    if config.agent.firewall.enabled
        && let Some(utility) = config.agent.firewall.utility.file_name()
        && utility.to_string_lossy() == "iptables"
    {
        hidden_scripts = Some(format!(
            "### START OF HIDDEN SCRIPTS ###
PostUp = sudo sysctl -w net.ipv4.ip_forward=1
PostDown = sudo sysctl -w net.ipv4.ip_forward=0
PostUp = {fw_utility} -t nat -A POSTROUTING -s {subnet} -o {gateway} -j MASQUERADE;
PostDown = {fw_utility} -t nat -D POSTROUTING -s {subnet} -o {gateway} -j MASQUERADE;
PostUp = {fw_utility} -A INPUT -p udp -m udp --dport {port} -j ACCEPT;
PostDown = {fw_utility} -D INPUT -p udp -m udp --dport {port} -j ACCEPT;
PostUp = {fw_utility} -A FORWARD -i {interface} -j ACCEPT;
PostDown = {fw_utility} -D FORWARD -i {interface} -j ACCEPT;
PostUp = {fw_utility} -A FORWARD -o {interface} -j ACCEPT;
PostDown = {fw_utility} -D FORWARD -o {interface} -j ACCEPT;
### END OF HIDDEN SCRIPTS ###",
            fw_utility = config.agent.firewall.utility.to_string_lossy(),
            subnet = config.network.subnet,
            gateway = config.agent.vpn.gateway,
            port = config.agent.vpn.port,
            interface = config.network.identifier,
        ));
    }
    let wg_conf = match get_peer_wg_config(
        &config.network,
        config.network.this_peer.clone(),
        full_version!(),
        hidden_scripts,
    ) {
        Ok(n) => n,
        Err(e) => {
            return Err(WireGuardCommandError::Other(e.to_string()));
        }
    };

    // write the content to the .conf file
    let config_path = WIREGUARD_CONFIG_FILE.get().unwrap();
    let config_parent_path = config_path.parent().unwrap();
    // make sure the parent directory exists
    match fs::create_dir_all(config_parent_path) {
        Ok(_) => {}
        Err(e) => {
            return Err(WireGuardCommandError::FolderCreationError(
                PathBuf::from(config_parent_path),
                e,
            ));
        }
    };
    // open the file with write-only permissions
    let mut file = match File::create(config_path) {
        Ok(f) => f,
        Err(e) => {
            return Err(WireGuardCommandError::FileCreationError(
                config_path.clone(),
                e,
            ));
        }
    };
    // dump the new conf to the file
    match file.write_all(wg_conf.as_bytes()) {
        Ok(_) => {}
        Err(e) => {
            return Err(WireGuardCommandError::FileWriteError(
                config_path.clone(),
                e,
            ));
        }
    };
    Ok(())
}

pub(crate) fn start_tunnel(config: &Config) -> Result<(), WireGuardCommandError> {
    // override .conf from .yml
    update_conf_file(config)?;

    // disable then enable and ignore any error when disabling
    let _ = disable_tunnel(config);
    enable_tunnel(config)?;

    log::info!(
        "wireguard tunnel accessible at {}:{}",
        config.agent.web.address,
        config.agent.vpn.port
    );
    Ok(())
}

pub(crate) fn get_public_private_keys() -> Result<Value, WireGuardCommandError> {
    #[allow(unused_assignments)]
    let mut private_key = "".parse().unwrap();

    // wg genkey
    let readable_command = "$ wg genkey".to_string();
    match Command::new("wg").arg("genkey").output() {
        Ok(output) => {
            log::info!("{readable_command}");
            if !output.stdout.is_empty() {
                log::debug!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                log::warn!("{}", String::from_utf8_lossy(&output.stderr));
            }
            if !output.status.success() {
                return Err(WireGuardCommandError::CommandExecNotSuccessful(
                    readable_command,
                ));
            }
            private_key = String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
        Err(e) => {
            return Err(WireGuardCommandError::CommandExecError(readable_command, e));
        }
    };

    // wg genkey | wg pubkey
    let readable_command = "$ wg genkey | wg pubkey".to_string();
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
                        return Err(WireGuardCommandError::CommandExecError(readable_command, e));
                    }
                }
            } else {
                return Err(WireGuardCommandError::CommandExecError(
                    readable_command,
                    std::io::Error::other("not able to create a pipe"),
                ));
            }
            match child.wait_with_output() {
                Ok(output) => {
                    log::info!("{readable_command}");
                    if !output.stdout.is_empty() {
                        log::debug!("{}", String::from_utf8_lossy(&output.stdout));
                    }
                    if !output.stderr.is_empty() {
                        log::warn!("{}", String::from_utf8_lossy(&output.stderr));
                    }
                    if !output.status.success() {
                        return Err(WireGuardCommandError::CommandExecNotSuccessful(
                            readable_command,
                        ));
                    }
                    Ok(json!({
                        "private_key": private_key,
                        "public_key": String::from_utf8_lossy(&output.stdout).trim().to_string(),
                    }))
                }
                Err(e) => Err(WireGuardCommandError::CommandExecError(readable_command, e)),
            }
        }
        Err(e) => Err(WireGuardCommandError::CommandExecError(readable_command, e)),
    }
}

pub(crate) fn get_pre_shared_key() -> Result<Value, WireGuardCommandError> {
    // wg genpsk
    let readable_command = "$ wg genpsk".to_string();
    match Command::new("wg").arg("genpsk").output() {
        Ok(output) => {
            log::info!("{readable_command}");
            if !output.stdout.is_empty() {
                log::debug!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                log::warn!("{}", String::from_utf8_lossy(&output.stderr));
            }
            if !output.status.success() {
                return Err(WireGuardCommandError::CommandExecNotSuccessful(
                    readable_command,
                ));
            }
            Ok(
                json!({"pre_shared_key": String::from_utf8_lossy(&output.stdout).trim().to_string()}),
            )
        }
        Err(e) => Err(WireGuardCommandError::CommandExecError(readable_command, e)),
    }
}
