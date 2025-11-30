use crate::{WG_QUICKRS_CONFIG_FILE, WG_QUICKRS_CONFIG_FOLDER};
use crate::wireguard::cmd::{get_telemetry, status_tunnel};
use wg_quickrs_lib::types::config::{Config, ConfigFile, ConfigWNetworkDigest};
use wg_quickrs_lib::types::api::{Summary};
use wg_quickrs_lib::types::misc::{WireGuardStatus};
use wg_quickrs_lib::validation::config_file::{validate_config_file, ConfigFileValidationError};
use wg_quickrs_lib::validation::error::ValidationError;
use wg_quickrs_lib::macros::wg_quickrs_version;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{RwLock, OnceLock};
use chrono::Utc;
use thiserror::Error;
use semver::Version;

#[derive(Error, Debug)]
pub enum ConfUtilError {
    #[error("failed to write config file file at {0}: {1}")]
    Write(PathBuf, std::io::Error),
    #[error("failed to read config file at {0}: {1}")]
    Read(PathBuf, std::io::Error),
    #[error("invalid config file format: {0}")]
    Parse(serde_yml::Error),
    #[error("failed to serialize config object: {0}")]
    Serialization(serde_yml::Error),
    #[error("failed to encode digest: {0}")]
    DigestEncoding(base16ct::Error),
    #[error("failed to acquire lock: {0}")]
    MutexLockFailed(String),
    #[error("failed to set mutex variable")]
    MutexSetFailed(),
    #[error("invalid version semantic version: {0}")]
    InvalidVersion(semver::Error),
    #[error("version in conf.yml not supported: expected {0}, got {1}")]
    VersionNotSupported(String, String),
    #[error("validation Error: {0}")]
    Validation(#[from] ValidationError),
    #[error("{0}")]
    WireGuardLibError(#[from] wg_quickrs_lib::types::misc::WireGuardLibError),
    #[error("{0}")]
    ConfigFile(#[from] ConfigFileValidationError),
}

pub static CONFIG_W_NETWORK_DIGEST: OnceLock<RwLock<ConfigWNetworkDigest>> = OnceLock::new();

fn set_or_init_config_w_digest(config_w_network_digest: ConfigWNetworkDigest) -> Result<(), ConfUtilError> {
    let mut_opt = CONFIG_W_NETWORK_DIGEST.get();
    if mut_opt.is_none() {
        return CONFIG_W_NETWORK_DIGEST
            .set(RwLock::new(config_w_network_digest.clone()))
            .map_err(|_| ConfUtilError::MutexSetFailed());
    }

    mut_opt
        .unwrap()
        .write()
        .map(|mut c| {
            c.agent = config_w_network_digest.agent;
            c.network_w_digest = config_w_network_digest.network_w_digest;
            Ok(())
        })
        .map_err(|e| ConfUtilError::MutexLockFailed(e.to_string()))?
}

pub(crate) fn get_config() -> Result<Config, ConfUtilError> {
    let config_w_digest = get_config_w_digest()?;
    Ok(config_w_digest.to_config())
}

fn get_config_w_digest() -> Result<ConfigWNetworkDigest, ConfUtilError> {
    let mut_opt = CONFIG_W_NETWORK_DIGEST.get();
    if let Some(m) = mut_opt {
        return m
            .read()
            .map(|c| c.clone())
            .map_err(|e| ConfUtilError::MutexLockFailed(e.to_string()));
    }

    let config_file_path = WG_QUICKRS_CONFIG_FILE.get().unwrap();
    let config_str = fs::read_to_string(config_file_path)
        .map_err(|e| ConfUtilError::Read(config_file_path.clone(), e))?;
    let mut config_file: ConfigFile = serde_yml::from_str(&config_str).map_err(ConfUtilError::Parse)?;
    let build_version = Version::parse(wg_quickrs_version!()).unwrap();
    let conf_ver = Version::parse(config_file.version.as_str()).map_err(ConfUtilError::InvalidVersion)?;
    if build_version.major != conf_ver.major {
        return Err(ConfUtilError::VersionNotSupported(
            format!("{}.x.x", build_version.major),
            config_file.version
        ));
    }
    // Validate config_file fields
    let config_folder_path = WG_QUICKRS_CONFIG_FOLDER.get().unwrap();
    validate_config_file(&mut config_file, config_folder_path)?;

    let config_w_digest = ConfigWNetworkDigest::from_config(Config::from(&config_file))?;
    set_or_init_config_w_digest(config_w_digest.clone())?;
    log::debug!("loaded config file");
    Ok(config_w_digest)
}

pub(crate) fn get_summary() -> Result<Summary, ConfUtilError> {
    let config_w_digest = get_config_w_digest()?;
    let status = status_tunnel().unwrap_or_else(|e| {
        log::error!("{e}");
        WireGuardStatus::UNKNOWN
    });
    // let telemetry = None;
    let telemetry = if status == WireGuardStatus::UP {
        get_telemetry().unwrap_or_else(|e| {
            log::error!("{e}");
            None
        })
    } else {
        None
    };

    Ok(Summary {
        network: config_w_digest.network_w_digest.network,
        telemetry,
        digest: config_w_digest.network_w_digest.digest,
        status,
        timestamp: Utc::now(),
    })
}

pub(crate) fn set_config(config: &mut Config) -> Result<(), ConfUtilError> {
    let mut config_file = ConfigFile::from(&config.clone());
    let config_folder_path = WG_QUICKRS_CONFIG_FOLDER.get().unwrap();
    validate_config_file(&mut config_file, config_folder_path)?;

    let config_w_digest = ConfigWNetworkDigest::from_config(config.clone())?;
    set_or_init_config_w_digest(config_w_digest)?;

    let config_file_str = serde_yml::to_string(&config_file).map_err(ConfUtilError::Serialization)?;
    write_config(config_file_str)
}

pub(crate) fn write_config(config_file_str: String) -> Result<(), ConfUtilError> {
    let file_path = WG_QUICKRS_CONFIG_FILE.get().unwrap();
    let mut file = File::create(file_path).map_err(|e| ConfUtilError::Write(file_path.clone(), e))?;
    file.write_all(config_file_str.as_bytes())
        .map_err(|e| ConfUtilError::Write(file_path.clone(), e))?;

    log::info!("updated config file");
    Ok(())
}
