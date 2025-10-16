use crate::{WG_QUICKRS_CONFIG_FILE};
use crate::macros::*;
use crate::conf::timestamp;
use crate::wireguard::cmd::{get_telemetry, status_tunnel};
use wg_quickrs_wasm::types::{Config, Agent, Network, Summary, WireGuardStatus, ConfigFile};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use thiserror::Error;
use semver::{Version, VersionReq};


#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ConfigWNetworkDigest {
    pub agent: Agent,
    pub network_w_digest: NetworkWDigest,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct NetworkWDigest {
    pub network: Network,
    pub digest: String,
}

impl NetworkWDigest {
    pub(crate) fn from_network(
        network: Network,
    ) -> Result<Self, ConfUtilError> {
        let network_str = serde_yml::to_string(&network).map_err(ConfUtilError::Serialization)?;
        let mut buf = [0u8; 64];
        let digest = base16ct::lower::encode_str(&Sha256::digest(network_str.as_bytes()), &mut buf)
            .map(|d| d.to_string())
            .map_err(ConfUtilError::DigestEncoding)?;
        Ok(NetworkWDigest { network, digest })
    }
}

impl ConfigWNetworkDigest {
    pub(crate) fn from_config(config: Config) -> Result<Self, ConfUtilError> {
        let network_w_digest = NetworkWDigest::from_network(config.network)?;
        Ok(ConfigWNetworkDigest { agent: config.agent, network_w_digest })
    }

    pub(crate) fn to_config(self) -> Config {
        Config{ agent: self.agent, network: self.network_w_digest.network }
    }

    fn set_or_init(config_w_network_digest: ConfigWNetworkDigest) -> Result<(), ConfUtilError> {
        let mut_opt = CONFIG_W_NETWORK_DIGEST.get();
        if mut_opt.is_none() {
            return CONFIG_W_NETWORK_DIGEST
                .set(Mutex::new(config_w_network_digest.clone()))
                .map_err(|_| ConfUtilError::MutexSetFailed());
        }

        mut_opt
            .unwrap()
            .lock()
            .map(|mut c| {
                c.agent = config_w_network_digest.agent;
                c.network_w_digest = config_w_network_digest.network_w_digest;
                Ok(())
            })
            .map_err(|e| ConfUtilError::MutexLockFailed(e.to_string()))?
    }
}

pub static CONFIG_W_NETWORK_DIGEST: OnceLock<Mutex<ConfigWNetworkDigest>> = OnceLock::new();

#[derive(Error, Debug)]
pub enum ConfUtilError {
    #[error("conf::util::error::write -> failed to write config file file at {0}: {1}")]
    Write(PathBuf, std::io::Error),
    #[error("conf::util::error::read -> failed to read config file at {0}: {1}")]
    Read(PathBuf, std::io::Error),
    #[error("conf::util::error::parse -> invalid config file format: {0}")]
    Parse(serde_yml::Error),
    #[error("conf::util::error::serialization -> failed to serialize config object: {0}")]
    Serialization(serde_yml::Error),
    #[error("conf::util::error::digest_encoding -> failed to encode digest: {0}")]
    DigestEncoding(base16ct::Error),
    #[error("conf::util::error::mutex_lock_failed -> failed to acquire lock: {0}")]
    MutexLockFailed(String),
    #[error("conf::util::error::mutex_set_failed -> failed to set mutex variable")]
    MutexSetFailed(),
    #[error("conf::util::error::invalid_version -> invalid version semantic version: {0}")]
    InvalidVersion(semver::Error),
    #[error("conf::util::error::version_not_supported -> conf.yml version not supported: expected {0}, got {1}")]
    VersionNotSupported(String, String),
}

pub(crate) fn get_config() -> Result<Config, ConfUtilError> {
    let config_w_digest = get_config_w_digest()?;
    Ok(config_w_digest.to_config())
}

fn get_config_w_digest() -> Result<ConfigWNetworkDigest, ConfUtilError> {
    let mut_opt = CONFIG_W_NETWORK_DIGEST.get();
    if let Some(m) = mut_opt {
        return m
            .lock()
            .map(|c| c.clone())
            .map_err(|e| ConfUtilError::MutexLockFailed(e.to_string()));
    }

    let file_path = WG_QUICKRS_CONFIG_FILE.get().unwrap();
    let config_str =
        fs::read_to_string(file_path).map_err(|e| ConfUtilError::Read(file_path.clone(), e))?;
    let config_file: ConfigFile = serde_yml::from_str(&config_str).map_err(ConfUtilError::Parse)?;
    let build_version = Version::parse(wg_quickrs_version!()).unwrap();
    let version_req = VersionReq::parse(format!("={}", build_version.major).as_str()).unwrap();
    let conf_ver = Version::parse(config_file.version.as_str()).map_err(ConfUtilError::InvalidVersion)?;
    if !version_req.matches(&conf_ver) {
        return Err(ConfUtilError::VersionNotSupported(format!("{}.x.x", build_version.major), config_file.version));
    }
    
    let config_w_digest = ConfigWNetworkDigest::from_config(Config::from(&config_file))?;
    ConfigWNetworkDigest::set_or_init(config_w_digest.clone())?;
    log::info!("loaded config file");
    Ok(config_w_digest)
}

pub(crate) fn get_summary() -> Result<Summary, ConfUtilError> {
    let config_w_digest = get_config_w_digest()?;
    let status = status_tunnel().unwrap_or_else(|e| {
        log::error!("{e}");
        WireGuardStatus::UNKNOWN
    });
    // let telemetry = None;
    let telemetry = if status.value() == WireGuardStatus::UP.value() {
        get_telemetry().unwrap_or_else(|e| {
            log::error!("{e}");
            None
        })
    } else {
        None
    };
    let timestamp = timestamp::get_now_timestamp_formatted();

    Ok(Summary {
        network: config_w_digest.network_w_digest.network,
        telemetry,
        digest: config_w_digest.network_w_digest.digest,
        status: status.value(),
        timestamp,
    })
}

pub(crate) fn set_config(config: &mut Config) -> Result<(), ConfUtilError> {
    let config_w_digest = ConfigWNetworkDigest::from_config(config.clone())?;
    ConfigWNetworkDigest::set_or_init(config_w_digest.clone())?;

    let config_file = ConfigFile {
        version: wg_quickrs_version!().into(),
        agent: config.agent.clone(),
        network: config.network.clone(),
    };
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
