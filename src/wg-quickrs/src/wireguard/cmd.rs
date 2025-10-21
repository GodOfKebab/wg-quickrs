use crate::{conf, WIREGUARD_CONFIG_FILE};
use crate::macros::*;
use once_cell::sync::Lazy;
use wg_quickrs_wasm::helpers::get_peer_wg_config;
use wg_quickrs_wasm::types::{Config, Telemetry, TelemetryData, TelemetryDatum, WireGuardStatus};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::NamedTempFile;
use thiserror::Error;
use tokio::signal::unix::{signal, SignalKind};
use std::os::unix::process::CommandExt; // for before_exec

const TELEMETRY_CAPACITY: usize = 21; // 20 throughput measurements
const TELEMETRY_INTERVAL: u64 = 1000;
type TelemetryType = Lazy<Arc<Mutex<VecDeque<TelemetryData>>>>;
static TELEMETRY: TelemetryType =
    Lazy::new(|| Arc::new(Mutex::new(VecDeque::with_capacity(TELEMETRY_CAPACITY))));

static LAST_TELEMETRY_QUERY_TS: Lazy<Arc<Mutex<u64>>> = Lazy::new(|| Arc::new(Mutex::new(0)));

fn update_timestamp(ts: &Arc<Mutex<u64>>) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut ts = ts.lock().unwrap();
    *ts = now;
}

fn get_since_timestamp(ts: &Arc<Mutex<u64>>) -> u64 {
    let start = *ts.lock().unwrap();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    now.saturating_sub(start) * 1000
}

#[derive(Error, Debug)]
pub enum WireGuardCommandError {
    #[error("wireguard::cmd::error::mutex_lock_failed -> failed to acquire lock: {0}")]
    MutexLockFailed(String),
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
    #[error(
        "wireguard::cmd::error::interface_sync_failed -> failed to sync wireguard interface: {0}"
    )]
    InterfaceSyncFailed(String),
    #[error("wireguard::cmd::error::other -> unexpected error: {0}")]
    Other(String),
}

static WG_INTERFACE: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("".to_string()));
pub static WG_STATUS: Mutex<WireGuardStatus> = Mutex::new(WireGuardStatus::DOWN);

pub(crate) async fn run_vpn_server(
    config: &Config,
    wireguard_config_folder: &Path,
) -> std::io::Result<()> {
    WIREGUARD_CONFIG_FILE
        .set(wireguard_config_folder.join(format!("{}.conf", config.network.identifier)))
        .expect("Failed to set WIREGUARD_CONFIG_FILE");
    log::info!(
        "using the wireguard config file at \"{}\"",
        WIREGUARD_CONFIG_FILE.get().unwrap().display()
    );

    Box::pin(async move {
        // override .conf from .yml
        update_conf_file(config).unwrap_or_else(|e| {
            log::error!("Failed to update the wireguard config file: {e}");
        });

        log::info!("Always disable wireguard first on startup");
        let _ = disable_tunnel(config);
        match WG_STATUS.lock() {
            Ok(mut w) => { *w = WireGuardStatus::DOWN }
            Err(e) => log::error!("Failed to acquire lock when forcing internal wireguard status tracker to down: {e}")
        }
        if !config.agent.vpn.enabled {
            log::warn!("VPN server is disabled.");
        } else {
            enable_tunnel(config).unwrap_or_else(|e| {
                log::error!("Failed to enable the wireguard tunnel: {e}");
            });
        }

        let mut signal_terminate = signal(SignalKind::terminate()).unwrap();
        let mut signal_interrupt = signal(SignalKind::interrupt()).unwrap();
        let mut ticker = tokio::time::interval(Duration::from_millis(TELEMETRY_INTERVAL));
        tokio::select! {
            _ = async {
                loop {
                    ticker.tick().await;
                    run_loop();
                }
            } => {},
            _ = signal_terminate.recv() => log::info!("Received SIGTERM"),
            _ = signal_interrupt.recv() => log::info!("Received SIGINT"),
        }
        log::info!("Stopping the wireguard tunnel... (ignore errors if it fails to find interface because \"Backgrounding route monitor\" might have stopped wg already)");
        let _ = disable_tunnel(config);
        Ok(())
    })
    .await
}

fn run_loop() {
    // don't check if not up
    match WG_STATUS.lock() {
        Ok(status) => {
            if status.clone() != WireGuardStatus::UP {
                return;
            }
        }
        Err(e) => {
            log::error!("{}", WireGuardCommandError::MutexLockFailed(e.to_string()));
            return;
        }
    }

    // don't check telemetry if not queried recently (60 seconds)
    if get_since_timestamp(&LAST_TELEMETRY_QUERY_TS)
        > TELEMETRY_INTERVAL * TELEMETRY_CAPACITY as u64
    {
        return;
    }

    let config = match conf::util::get_config() {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            return;
        }
    };

    match show_dump(&config) {
        Ok(telemetry) => {
            let mut buf = TELEMETRY.lock().unwrap();
            if buf.len() == TELEMETRY_CAPACITY {
                buf.pop_front(); // remove oldest
            }
            buf.push_back(TelemetryData {
                datum: telemetry,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis(),
            }); // add newest
        }
        Err(e) => log::error!("Failed to get telemetry data => {}", e),
    }
}

pub(crate) fn get_telemetry() -> Result<Option<Telemetry>, WireGuardCommandError> {
    // if the last telem data is old, reset the buffer
    if get_since_timestamp(&LAST_TELEMETRY_QUERY_TS)
        > TELEMETRY_INTERVAL * TELEMETRY_CAPACITY as u64
    {
        let mut buf = TELEMETRY.lock().unwrap();
        *buf = VecDeque::with_capacity(TELEMETRY_CAPACITY);
    }
    update_timestamp(&LAST_TELEMETRY_QUERY_TS);

    match TELEMETRY.lock() {
        Ok(buf) => Ok(Some(Telemetry {
            max_len: TELEMETRY_CAPACITY as u8,
            data: buf.iter().cloned().collect(),
        })),
        Err(e) => Err(WireGuardCommandError::MutexLockFailed(e.to_string())),
    }
}

pub(crate) fn status_tunnel() -> Result<WireGuardStatus, WireGuardCommandError> {
    let wg_status_mut = WG_STATUS
        .lock()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;
    Ok(wg_status_mut.clone())
}

fn show_dump(config: &Config) -> Result<HashMap<String, TelemetryDatum>, WireGuardCommandError> {
    let wg_interface_mut = WG_INTERFACE
        .lock()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;

    if (*wg_interface_mut).is_empty() {
        return Err(WireGuardCommandError::InterfaceMissing);
    }

    // sudo wg show INTERFACE dump
    let readable_command = format!("$ sudo wg show {} dump", &*wg_interface_mut);
    log::info!("{readable_command}");
    match Command::new("sudo")
        .arg("wg")
        .arg("show")
        .arg(&*wg_interface_mut)
        .arg("dump")
        .output()
    {
        Ok(output) => {
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
                    if wg_quickrs_wasm::helpers::wg_public_key_from_private_key(&peer_details.private_key).ok().as_deref() != Some(public_key) {
                        continue;
                    }
                    let transfer_rx = parts[5].parse::<u64>().unwrap_or(0);
                    let transfer_tx = parts[6].parse::<u64>().unwrap_or(0);
                    let connection_id =
                        wg_quickrs_wasm::helpers::get_connection_id(&config.network.this_peer, &peer_id);

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

    let wg_interface_mut = WG_INTERFACE
        .lock()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;

    let wg_conf_stripped = match get_peer_wg_config(
        &config.network,
        config.network.this_peer.clone(),
        full_version!(),
        true,
        None,
    ) {
        Ok(n) => n,
        Err(e) => {
            return Err(WireGuardCommandError::Other(e.to_string()));
        }
    };

    // Write to a temp file
    let mut temp = match NamedTempFile::new() {
        Ok(file) => file,
        Err(e) => {
            return Err(WireGuardCommandError::Other(e.to_string()));
        }
    };

    match temp.write_all((&wg_conf_stripped).as_ref()) {
        Ok(_) => {}
        Err(e) => {
            return Err(WireGuardCommandError::FileWriteError(
                PathBuf::from(temp.path()),
                e,
            ));
        }
    };
    let temp_path = temp.path().to_owned(); // Save path before dropping

    // wg syncconf WG_INTERFACE <(WG_CONF_STRIPPED)
    let readable_command = format!(
        "$ wg syncconf {} <(WG_CONF_STRIPPED)",
        &*wg_interface_mut,
    );
    log::info!("{readable_command}");
    match Command::new("sudo")
        .arg("wg")
        .arg("syncconf")
        .arg(&*wg_interface_mut)
        .arg(temp_path)
        .output()
    {
        Ok(output) => {
            if !output.stdout.is_empty() {
                log::debug!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                log::warn!("{}", String::from_utf8_lossy(&output.stderr));
            }
            if !output.status.success() {
                return Err(WireGuardCommandError::InterfaceSyncFailed(readable_command));
            }
            Ok(())
        }
        Err(e) => Err(WireGuardCommandError::CommandExecError(readable_command, e)),
    }
}

pub(crate) fn disable_tunnel(config: &Config) -> Result<(), WireGuardCommandError> {
    *WG_STATUS
        .lock()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))? =
        WireGuardStatus::UNKNOWN;
    *WG_INTERFACE
        .lock()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))? = String::new();

    // sudo wg-quick down INTERFACE
    let readable_command = format!("$ sudo wg-quick down {}", config.network.identifier.clone());
    log::info!("{readable_command}");
    match Command::new("sudo")
        .arg("wg-quick")
        .arg("down")
        .arg(config.network.identifier.clone())
        .output()
    {
        Ok(output) => {
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
            *WG_STATUS
                .lock()
                .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))? =
                WireGuardStatus::DOWN;
            // reset the telemetry
            let mut buf = TELEMETRY.lock().unwrap();
            *buf = VecDeque::with_capacity(TELEMETRY_CAPACITY);
            Ok(())
        }
        Err(e) => Err(WireGuardCommandError::CommandExecError(readable_command, e)),
    }
}

pub(crate) fn enable_tunnel(config: &Config) -> Result<(), WireGuardCommandError> {
    *WG_STATUS
        .lock()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))? =
        WireGuardStatus::UNKNOWN;

    // sudo wg-quick up INTERFACE
    let readable_command = format!("$ sudo wg-quick up {}", config.network.identifier.clone());
    log::info!("{readable_command}");
    unsafe {
        match Command::new("sudo")
            .arg("wg-quick")
            .arg("up")
            .arg(config.network.identifier.clone())
            .pre_exec(|| {
                // Create a new session -> child will NOT be in parent’s process group
                libc::setsid(); // unsafe function
                Ok(())
            })
            .output()
        {
            Ok(output) => {
                if !output.stdout.is_empty() {
                    log::debug!("{}", String::from_utf8_lossy(&output.stdout));
                }
                if !output.stderr.is_empty() {
                    log::warn!("{}", String::from_utf8_lossy(&output.stderr));
                }
                if !output.stderr.is_empty() {
                    let mut wg_interface_mut = WG_INTERFACE
                        .lock()
                        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;

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
                    *WG_STATUS
                        .lock()
                        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))? =
                        WireGuardStatus::UP;
                }
                if !output.status.success() {
                    return Err(WireGuardCommandError::CommandExecNotSuccessful(
                        readable_command,
                    ));
                }
                log::info!(
                    "Started the wireguard tunnel at {}:{}",
                    config.agent.web.address,
                    config.agent.vpn.port
                );
                Ok(())
            }
            Err(e) => Err(WireGuardCommandError::CommandExecError(readable_command, e)),
        }
    }
}

pub(crate) fn update_conf_file(config: &Config) -> Result<(), WireGuardCommandError> {
    // generate .conf content with hidden scripts
    let mut hidden_scripts = None;
    if config.agent.firewall.enabled
        && let Some(utility) = config.agent.firewall.utility.file_name() {
        if utility.to_string_lossy() == "iptables" {
            let mut _hidden_scripts = format!(
                "### START OF HIDDEN SCRIPTS ###
PostUp = {fw_utility} -t nat -A POSTROUTING -s {subnet} -o {gateway} -j MASQUERADE;
PostDown = {fw_utility} -t nat -D POSTROUTING -s {subnet} -o {gateway} -j MASQUERADE;
PostUp = {fw_utility} -A INPUT -p udp -m udp --dport {port} -j ACCEPT;
PostDown = {fw_utility} -D INPUT -p udp -m udp --dport {port} -j ACCEPT;
PostUp = {fw_utility} -A FORWARD -i {interface} -j ACCEPT;
PostDown = {fw_utility} -D FORWARD -i {interface} -j ACCEPT;
PostUp = {fw_utility} -A FORWARD -o {interface} -j ACCEPT;
PostDown = {fw_utility} -D FORWARD -o {interface} -j ACCEPT;",
                fw_utility = config.agent.firewall.utility.to_string_lossy(),
                subnet = config.network.subnet,
                gateway = config.agent.firewall.gateway,
                port = config.agent.vpn.port,
                interface = config.network.identifier,
            );
            #[cfg(not(feature = "docker"))]
            {
                _hidden_scripts.push_str(
                    "\nPostUp = sysctl -w net.ipv4.ip_forward=1;\n\
                PostDown = sysctl -w net.ipv4.ip_forward=0;",
                );
            }
            _hidden_scripts.push_str("\n### END OF HIDDEN SCRIPTS ###\n");
            hidden_scripts = Some(_hidden_scripts);
        } else if utility.to_string_lossy() == "pfctl" {
            // add the following nat rule to pf.conf
            // check for a line that starts with "nat" and paste the nat_rule after it because pf.conf requires the rules to be in order
            let nat_rule = format!("nat on {gateway} from {subnet} to any -> {gateway}",
                gateway = config.agent.firewall.gateway,
                subnet = config.network.subnet);
            hidden_scripts = Some(format!(
                "### START OF HIDDEN SCRIPTS ###
PostUp = awk \"/^nat/ {{print; print \\\"{nat_rule}\\\"; next}}1\" /etc/pf.conf > /etc/pf.conf.new && mv /etc/pf.conf /etc/pf.conf.bak && mv /etc/pf.conf.new /etc/pf.conf;
PostUp = grep -qxF '{nat_rule}' /etc/pf.conf || echo '*** could NOT configure firewall because there are no existing NAT rules. See notes at docs/MACOS-FIREWALL.md ' >&2;
PostUp = grep -qxF '{nat_rule}' /etc/pf.conf || exit 1;
PostDown = awk -v line='{nat_rule}' '$0 != line' /etc/pf.conf > /etc/pf.conf.new && mv /etc/pf.conf /etc/pf.conf.bak && mv /etc/pf.conf.new /etc/pf.conf;
PostUp = {fw_utility} -f /etc/pf.conf;
PostUp = {fw_utility} -e || true;
PostDown = {fw_utility} -d || true;
PostUp = sysctl -w net.inet.ip.forwarding=1;
PostDown = sysctl -w net.inet.ip.forwarding=0;
### END OF HIDDEN SCRIPTS ###\n",
                fw_utility = config.agent.firewall.utility.to_string_lossy(),
            ));
        }
    }
    let wg_conf = match get_peer_wg_config(
        &config.network,
        config.network.this_peer.clone(),
        full_version!(),
        false,
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

