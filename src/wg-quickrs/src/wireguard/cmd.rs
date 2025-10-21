use crate::{conf};
use crate::macros::*;
use once_cell::sync::Lazy;
use wg_quickrs_wasm::helpers::get_peer_wg_config;
use wg_quickrs_wasm::types::{Config, Telemetry, TelemetryData, TelemetryDatum, WireGuardStatus};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::NamedTempFile;
use thiserror::Error;
use tokio::signal::unix::{signal, SignalKind};
use crate::wireguard::wg_quick;

const TELEMETRY_CAPACITY: usize = 21;
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
    #[error("wireguard::cmd::error::command_exec_not_successful -> command for {0} completed unsuccessfully")]
    CommandExecNotSuccessful(String),
    #[error("wireguard::cmd::error::folder_creation_error -> failed to create folder at {0} failed: {1}")]
    FolderCreationError(PathBuf, std::io::Error),
    #[error("wireguard::cmd::error::file_creation_error -> failed to create file at {0} failed: {1}")]
    FileCreationError(PathBuf, std::io::Error),
    #[error("wireguard::cmd::error::file_write_error -> failed to write file at {0} failed: {1}")]
    FileWriteError(PathBuf, std::io::Error),
    #[error("wireguard::cmd::error::interface_sync_failed -> failed to sync wireguard interface: {0}")]
    InterfaceSyncFailed(String),
    #[error("wireguard::cmd::error::tunnel_error -> tunnel operation failed: {0}")]
    TunnelError(String),
    #[error("wireguard::cmd::error::other -> unexpected error: {0}")]
    Other(String),
}

impl From<wg_quick::TunnelError> for WireGuardCommandError {
    fn from(err: wg_quick::TunnelError) -> Self {
        WireGuardCommandError::TunnelError(err.to_string())
    }
}

static WG_INTERFACE: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("".to_string()));
pub static WG_STATUS: Mutex<WireGuardStatus> = Mutex::new(WireGuardStatus::DOWN);

pub(crate) async fn run_vpn_server(
    config: &Config,
    _wireguard_config_folder: &Path,
) -> std::io::Result<()> {
    log::info!("Starting VPN server with wg-quick native implementation");

    Box::pin(async move {
        log::info!("Always disable wireguard first on startup");
        let _ = disable_tunnel(config);
        match WG_STATUS.lock() {
            Ok(mut w) => *w = WireGuardStatus::DOWN,
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

        log::info!("Stopping the wireguard tunnel...");
        let _ = disable_tunnel(config);
        Ok(())
    })
        .await
}

fn run_loop() {
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
                buf.pop_front();
            }
            buf.push_back(TelemetryData {
                datum: telemetry,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis(),
            });
        }
        Err(e) => log::error!("Failed to get telemetry data => {}", e),
    }
}

pub(crate) fn get_telemetry() -> Result<Option<Telemetry>, WireGuardCommandError> {
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

    let readable_command = format!("$ wg show {} dump", &*wg_interface_mut);
    log::debug!("{readable_command}");

    match Command::new("wg")
        .arg("show")
        .arg(&*wg_interface_mut)
        .arg("dump")
        .output()
    {
        Ok(output) => {
            if !output.stdout.is_empty() {
                log::trace!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                log::debug!("{}", String::from_utf8_lossy(&output.stderr));
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
                    if wg_quickrs_wasm::helpers::wg_public_key_from_private_key(&peer_details.private_key)
                        .ok()
                        .as_deref() != Some(public_key)
                    {
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
                            transfer_a_to_b: if connection_id.starts_with(&format!("{}*", config.network.this_peer)) {
                                transfer_tx
                            } else {
                                transfer_rx
                            },
                            transfer_b_to_a: if connection_id.starts_with(&format!("{}*", config.network.this_peer)) {
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
    let temp_path = temp.path().to_owned();

    let readable_command = format!("$ wg syncconf {} <(WG_CONF_STRIPPED)", &*wg_interface_mut);
    log::info!("{readable_command}");

    match Command::new("wg")
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

    log::info!("Stopping WireGuard tunnel: {}", config.network.identifier);

    match wg_quick::stop_tunnel(config.clone()) {
        Ok(_) => {
            *WG_STATUS
                .lock()
                .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))? =
                WireGuardStatus::DOWN;

            let mut buf = TELEMETRY.lock().unwrap();
            *buf = VecDeque::with_capacity(TELEMETRY_CAPACITY);

            log::info!("WireGuard tunnel stopped successfully");
            Ok(())
        }
        Err(e) => {
            log::warn!("Failed to stop tunnel (may already be down): {}", e);
            *WG_STATUS
                .lock()
                .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))? =
                WireGuardStatus::DOWN;
            Ok(())
        }
    }
}

pub(crate) fn enable_tunnel(config: &Config) -> Result<(), WireGuardCommandError> {
    *WG_STATUS
        .lock()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))? =
        WireGuardStatus::UNKNOWN;

    log::info!("Starting WireGuard tunnel: {}", config.network.identifier);

    match wg_quick::start_tunnel(config.clone()) {
        Ok(real_interface) => {
            let mut wg_interface_mut = WG_INTERFACE
                .lock()
                .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;

            *wg_interface_mut = real_interface.clone();

            *WG_STATUS
                .lock()
                .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))? =
                WireGuardStatus::UP;

            log::info!(
                "Started WireGuard tunnel at {}:{} (interface: {})",
                config.agent.web.address,
                config.agent.vpn.port,
                real_interface
            );
            Ok(())
        }
        Err(e) => Err(WireGuardCommandError::from(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_config() -> Config {
        let mut peers = HashMap::new();

        let private_key = wg_quickrs_wasm::helpers::wg_generate_key();

        peers.insert(
            "peer1".to_string(),
            wg_quickrs_wasm::types::Peer {
                name: "Test Server".to_string(),
                address: "10.0.0.1/24".to_string(),
                endpoint: wg_quickrs_wasm::types::EnabledValue {
                    enabled: true,
                    value: "0.0.0.0:51820".to_string(),
                },
                kind: "server".to_string(),
                icon: wg_quickrs_wasm::types::EnabledValue::default(),
                dns: wg_quickrs_wasm::types::EnabledValue::default(),
                mtu: wg_quickrs_wasm::types::EnabledValue::default(),
                scripts: wg_quickrs_wasm::types::Scripts::default(),
                private_key,
                created_at: "2025-01-01T00:00:00Z".to_string(),
                updated_at: "2025-01-01T00:00:00Z".to_string(),
            },
        );

        Config {
            agent: wg_quickrs_wasm::types::Agent {
                web: wg_quickrs_wasm::types::AgentWeb {
                    address: "127.0.0.1".to_string(),
                    http: wg_quickrs_wasm::types::AgentWebHttp {
                        enabled: false,
                        port: 8080,
                    },
                    https: wg_quickrs_wasm::types::AgentWebHttps {
                        enabled: false,
                        port: 8443,
                        tls_cert: PathBuf::from("/dev/null"),
                        tls_key: PathBuf::from("/dev/null"),
                    },
                    password: wg_quickrs_wasm::types::Password {
                        enabled: false,
                        hash: "".to_string(),
                    },
                },
                vpn: wg_quickrs_wasm::types::AgentVpn {
                    enabled: true,
                    port: 51820,
                },
                firewall: wg_quickrs_wasm::types::AgentFirewall {
                    enabled: false,
                    utility: PathBuf::from("/usr/bin/ufw"),
                    gateway: "192.168.1.1".to_string(),
                },
            },
            network: wg_quickrs_wasm::types::Network {
                identifier: "wgtest_cmd".to_string(),
                subnet: "10.0.0.0/24".to_string(),
                this_peer: "peer1".to_string(),
                peers,
                connections: HashMap::new(),
                defaults: wg_quickrs_wasm::types::Defaults::default(),
                reservations: HashMap::new(),
                updated_at: "2025-01-01T00:00:00Z".to_string(),
            },
        }
    }

    #[test]
    #[ignore]
    fn test_enable_disable_tunnel() {
        let config = create_test_config();

        // Ensure clean state
        let _ = disable_tunnel(&config);

        // Test enable
        let result = enable_tunnel(&config);
        assert!(result.is_ok(), "Failed to enable tunnel: {:?}", result.err());

        // Check status
        let status = status_tunnel().unwrap();
        assert_eq!(status, WireGuardStatus::UP);

        // Check interface is set
        let interface = WG_INTERFACE.lock().unwrap();
        assert!(!interface.is_empty(), "Interface name not set");
        drop(interface);

        std::thread::sleep(std::time::Duration::from_secs(2));

        // Test disable
        let result = disable_tunnel(&config);
        assert!(result.is_ok(), "Failed to disable tunnel: {:?}", result.err());

        // Check status
        let status = status_tunnel().unwrap();
        assert_eq!(status, WireGuardStatus::DOWN);
    }

    #[test]
    #[ignore]
    fn test_telemetry_collection() {
        let config = create_test_config();

        let _ = disable_tunnel(&config);
        enable_tunnel(&config).unwrap();

        std::thread::sleep(std::time::Duration::from_secs(3));

        // Trigger telemetry collection
        update_timestamp(&LAST_TELEMETRY_QUERY_TS);
        run_loop();

        std::thread::sleep(std::time::Duration::from_secs(2));

        let telemetry = get_telemetry().unwrap();
        assert!(telemetry.is_some());

        disable_tunnel(&config).unwrap();
    }
}