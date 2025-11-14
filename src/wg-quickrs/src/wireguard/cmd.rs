use crate::{conf};
use once_cell::sync::Lazy;
use wg_quickrs_lib::helpers::get_peer_wg_config;
use wg_quickrs_lib::types::config::{Config};
use wg_quickrs_lib::types::api::{Telemetry, TelemetryData, TelemetryDatum};
use wg_quickrs_lib::types::misc::{WireGuardStatus};
use std::collections::{BTreeMap, VecDeque};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::Utc;
use tempfile::NamedTempFile;
use thiserror::Error;
use tokio::signal::unix::{signal, SignalKind};
use wg_quickrs_lib::types::network::ConnectionId;
use crate::helpers::{shell_cmd, ShellError};
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
    #[error("{0}")]
    ShellError(#[from] ShellError),
    #[error("wireguard::cmd::error::file_write_error -> failed to write file at {0} failed: {1}")]
    FileWriteError(PathBuf, std::io::Error),
    #[error("Failed to sync wireguard interface")]
    InterfaceSyncFailed(),
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

static WG_TUNNEL_MANAGER: Lazy<Mutex<wg_quick::TunnelManager>> = Lazy::new(|| Mutex::new(wg_quick::TunnelManager::new(Default::default())));
static WG_INTERFACE: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("".to_string()));
pub static WG_STATUS: Mutex<WireGuardStatus> = Mutex::new(WireGuardStatus::DOWN);

pub(crate) async fn run_vpn_server(
    config: &Config,
) -> std::io::Result<()> {
    if !config.agent.vpn.enabled {
        log::warn!("WireGuard tunnel is disabled");
        return Ok(());
    }
    log::info!("Starting WireGuard tunnel...");
    *WG_TUNNEL_MANAGER.lock().unwrap() = wg_quick::TunnelManager::new(Some(config.clone()));

    Box::pin(async move {
        let _ = disable_tunnel();
        match WG_STATUS.lock() {
            Ok(mut w) => *w = WireGuardStatus::DOWN,
            Err(e) => log::error!("Failed to acquire lock when forcing internal wireguard status tracker to down: {e}")
        }

        if !config.agent.vpn.enabled {
            log::warn!("VPN server is disabled.");
        } else {
            enable_tunnel().unwrap_or_else(|e| {
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

        let _ = disable_tunnel();
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
                timestamp: Utc::now().naive_utc(),
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

fn show_dump(config: &Config) -> Result<BTreeMap<ConnectionId, TelemetryDatum>, WireGuardCommandError> {
    let wg_interface_mut = WG_INTERFACE
        .lock()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;

    if (*wg_interface_mut).is_empty() {
        return Err(WireGuardCommandError::InterfaceMissing);
    }

    let output = shell_cmd(&["wg", "show", &*wg_interface_mut, "dump"])?;
    let mut telemetry = BTreeMap::<ConnectionId, TelemetryDatum>::new();

    let dump = String::from_utf8_lossy(&output.stdout);
    for line in dump.trim().lines().skip(1) {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 8 {
            continue;
        }
        let public_key = parts[0];

        for (peer_id, peer_details) in config.network.peers.clone() {
            if wg_quickrs_lib::helpers::wg_public_key_from_private_key(&peer_details.private_key).to_base64() != public_key
            {
                continue;
            }

            let transfer_rx = parts[5].parse::<u64>().unwrap_or(0);
            let transfer_tx = parts[6].parse::<u64>().unwrap_or(0);
            let connection_id =
                wg_quickrs_lib::helpers::get_connection_id(config.network.this_peer, peer_id);

            let (transfer_a_to_b, transfer_b_to_a) = if connection_id.a == config.network.this_peer {
                (transfer_tx, transfer_rx)
            } else {
                (transfer_rx, transfer_tx)
            };

            telemetry.insert(
                connection_id.clone(),
                TelemetryDatum {
                    latest_handshake_at: parts[4].parse::<u64>().unwrap_or(0),
                    transfer_a_to_b,
                    transfer_b_to_a,
                },
            );
            break;
        }
    }
    Ok(telemetry)
}

pub(crate) fn sync_conf(config: &Config) -> Result<(), WireGuardCommandError> {
    let wg_interface_mut = WG_INTERFACE
        .lock()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;

    let mut wg_tunnel_manager = WG_TUNNEL_MANAGER
        .lock()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;

    wg_tunnel_manager.config = Some(config.clone());

    let wg_conf_stripped = get_peer_wg_config(&config.network, &config.network.this_peer, true)
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;

    let mut temp = NamedTempFile::new()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;

    temp.write_all(wg_conf_stripped.as_ref())
        .map_err(|e| WireGuardCommandError::FileWriteError(PathBuf::from(temp.path()), e))?;

    let temp_path = temp.path().to_owned();
    let temp_path_str = temp_path.to_str()
        .ok_or_else(|| WireGuardCommandError::Other("Temporary file path contains invalid UTF-8".to_string()))?;

    let _ = shell_cmd(&["wg", "syncconf", &*wg_interface_mut, temp_path_str])
        .map_err(|_| WireGuardCommandError::InterfaceSyncFailed())?;
    Ok(())
}

pub(crate) fn disable_tunnel() -> Result<(), WireGuardCommandError> {
    *WG_STATUS
        .lock()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))? =
        WireGuardStatus::UNKNOWN;
    *WG_INTERFACE
        .lock()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))? = String::new();

    let mut wg_tunnel_manager = WG_TUNNEL_MANAGER
        .lock()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;

    match wg_tunnel_manager.stop_tunnel() {
        Ok(_) => {
            *WG_STATUS
                .lock()
                .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))? =
                WireGuardStatus::DOWN;

            let mut buf = TELEMETRY.lock().unwrap();
            *buf = VecDeque::with_capacity(TELEMETRY_CAPACITY);

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

pub(crate) fn enable_tunnel() -> Result<(), WireGuardCommandError> {
    *WG_STATUS
        .lock()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))? =
        WireGuardStatus::UNKNOWN;

    let mut wg_tunnel_manager = WG_TUNNEL_MANAGER
        .lock()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;

    match wg_tunnel_manager.start_tunnel() {
        Ok(real_interface) => {
            let mut wg_interface_mut = WG_INTERFACE
                .lock()
                .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;

            *wg_interface_mut = real_interface.clone();

            *WG_STATUS
                .lock()
                .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))? =
                WireGuardStatus::UP;
            Ok(())
        }
        Err(e) => Err(WireGuardCommandError::from(e)),
    }
}
