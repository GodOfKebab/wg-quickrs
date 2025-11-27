use crate::{conf};
use once_cell::sync::Lazy;
use wg_quickrs_lib::helpers::get_peer_wg_config;
use wg_quickrs_lib::types::config::{Config};
use wg_quickrs_lib::types::api::{Telemetry, TelemetryData, TelemetryDatum};
use wg_quickrs_lib::types::misc::{WireGuardStatus};
use std::collections::{BTreeMap, VecDeque};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
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
type TelemetryType = Lazy<Arc<RwLock<VecDeque<TelemetryData>>>>;
static TELEMETRY: TelemetryType =
    Lazy::new(|| Arc::new(RwLock::new(VecDeque::with_capacity(TELEMETRY_CAPACITY))));

static LAST_TELEMETRY_QUERY_TS: Lazy<Arc<RwLock<u64>>> = Lazy::new(|| Arc::new(RwLock::new(0)));

fn update_timestamp(ts: &Arc<RwLock<u64>>) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut ts = ts.write().unwrap();
    *ts = now;
}

fn get_since_timestamp(ts: &Arc<RwLock<u64>>) -> u64 {
    let start = *ts.read().unwrap();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    now.saturating_sub(start) * 1000
}

#[derive(Error, Debug)]
pub enum WireGuardCommandError {
    #[error("failed to acquire lock: {0}")]
    MutexLockFailed(String),
    #[error("WireGuard interface does not exist")]
    InterfaceMissing,
    #[error("{0}")]
    ShellError(#[from] ShellError),
    #[error("failed to write file at {0} failed: {1}")]
    FileWriteError(PathBuf, std::io::Error),
    #[error("failed to sync WireGuard interface")]
    InterfaceSyncFailed(),
    #[error("tunnel operation failed: {0}")]
    TunnelError(#[from] wg_quick::TunnelError),
}

static WG_TUNNEL_MANAGER: Lazy<RwLock<wg_quick::TunnelManager>> = Lazy::new(|| RwLock::new(wg_quick::TunnelManager::new(Default::default())));
pub static WG_STATUS: RwLock<WireGuardStatus> = RwLock::new(WireGuardStatus::UNKNOWN);

pub(crate) async fn run_vpn_server(
    config: &Config,
) -> std::io::Result<()> {
    if !config.agent.vpn.enabled {
        log::warn!("WireGuard tunnel is disabled");
        return Ok(());
    }
    let mut tunnel_manager = WG_TUNNEL_MANAGER.write().unwrap();
    tunnel_manager.config = Some(config.clone());
    drop(tunnel_manager);

    Box::pin(async move {
        let _ = disable_tunnel();

        log::info!("Starting WireGuard tunnel...");
        enable_tunnel().unwrap_or_else(|e| {
            log::error!("Failed to enable the wireguard tunnel: {e}");
        });

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
    match WG_STATUS.read() {
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
            let mut buf = TELEMETRY.write().unwrap();
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
        *TELEMETRY.write().unwrap() = VecDeque::with_capacity(TELEMETRY_CAPACITY);
    }
    update_timestamp(&LAST_TELEMETRY_QUERY_TS);

    match TELEMETRY.read() {
        Ok(buf) => Ok(Some(Telemetry {
            max_len: TELEMETRY_CAPACITY as u8,
            data: buf.iter().cloned().collect(),
        })),
        Err(e) => Err(WireGuardCommandError::MutexLockFailed(e.to_string())),
    }
}

pub(crate) fn status_tunnel() -> Result<WireGuardStatus, WireGuardCommandError> {
    let wg_status = WG_STATUS
        .read()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;
    Ok(wg_status.clone())
}

fn show_dump(config: &Config) -> Result<BTreeMap<ConnectionId, TelemetryDatum>, WireGuardCommandError> {
    let tunnel_manager = WG_TUNNEL_MANAGER
        .read()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;

    let real_interface = tunnel_manager.real_interface.as_ref().ok_or(WireGuardCommandError::InterfaceMissing)?;

    let wg = config.agent.vpn.wg.to_str().unwrap();
    let output = shell_cmd(&[wg, "show", real_interface, "dump"])?;
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
    let mut tunnel_manager = WG_TUNNEL_MANAGER
        .write()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;

    tunnel_manager.config = Some(config.clone());

    let wg_conf_stripped = get_peer_wg_config(&config.network, &config.network.this_peer, true)
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;

    let mut temp = NamedTempFile::new()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;

    temp.write_all(wg_conf_stripped.as_ref())
        .map_err(|e| WireGuardCommandError::FileWriteError(PathBuf::from(temp.path()), e))?;

    let temp_path = temp.path().to_owned();
    let temp_path_str = temp_path.to_str().unwrap();

    let wg = config.agent.vpn.wg.to_str().unwrap();
    let _ = shell_cmd(&[wg, "syncconf", tunnel_manager.real_interface.as_ref().unwrap(), temp_path_str])
        .map_err(|_| WireGuardCommandError::InterfaceSyncFailed())?;
    Ok(())
}

pub(crate) fn disable_tunnel() -> Result<(), WireGuardCommandError> {
    *WG_STATUS
        .write()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))? =
        WireGuardStatus::UNKNOWN;

    let mut tunnel_manager = WG_TUNNEL_MANAGER
        .write()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;

    tunnel_manager.stop_tunnel()?;
        *WG_STATUS
            .write()
            .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))? =
            WireGuardStatus::DOWN;

        *TELEMETRY.write().unwrap() = VecDeque::with_capacity(TELEMETRY_CAPACITY);

        Ok(())
}

pub(crate) fn enable_tunnel() -> Result<(), WireGuardCommandError> {
    *WG_STATUS
        .write()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))? =
        WireGuardStatus::UNKNOWN;

    let mut tunnel_manager = WG_TUNNEL_MANAGER
        .write()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))?;

    tunnel_manager.start_tunnel()?;
    *WG_STATUS
        .write()
        .map_err(|e| WireGuardCommandError::MutexLockFailed(e.to_string()))? =
        WireGuardStatus::UP;
    Ok(())
}
