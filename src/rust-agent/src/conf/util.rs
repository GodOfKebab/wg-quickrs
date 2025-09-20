use crate::WG_QUICKRS_CONFIG_FILE;
use crate::conf::timestamp;
use crate::wireguard::cmd::{get_telemetry, status_tunnel};
use rust_wasm::types::{Config, Summary, WireGuardStatus};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use thiserror::Error;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ConfigWDigest {
    pub config: Config,
    pub digest: String,
}

impl ConfigWDigest {
    pub(crate) fn from_config_w_str(
        config: Config,
        config_str: String,
    ) -> Result<Self, ConfUtilError> {
        let mut buf = [0u8; 64];
        let digest = base16ct::lower::encode_str(&Sha256::digest(config_str.as_bytes()), &mut buf)
            .map(|d| d.to_string())
            .map_err(ConfUtilError::DigestEncoding)?;

        Ok(ConfigWDigest { config, digest })
    }

    fn set_or_init(config_w_digest: ConfigWDigest) -> Result<(), ConfUtilError> {
        let mut_opt = CONFIG_W_DIGEST.get();
        if mut_opt.is_none() {
            return CONFIG_W_DIGEST
                .set(Mutex::new(config_w_digest.clone()))
                .map_err(|_| ConfUtilError::MutexSetFailed());
        }

        mut_opt
            .unwrap()
            .lock()
            .map(|mut c| {
                c.config = config_w_digest.config;
                c.digest = config_w_digest.digest;
                Ok(())
            })
            .map_err(|e| ConfUtilError::MutexLockFailed(e.to_string()))?
    }
}

pub static CONFIG_W_DIGEST: OnceLock<Mutex<ConfigWDigest>> = OnceLock::new();

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
}

pub(crate) fn get_config() -> Result<Config, ConfUtilError> {
    let config_w_digest: ConfigWDigest = get_config_w_digest()?;
    Ok(config_w_digest.config)
}

fn get_config_w_digest() -> Result<ConfigWDigest, ConfUtilError> {
    let mut_opt = CONFIG_W_DIGEST.get();
    if let Some(m) = mut_opt {
        return m
            .lock()
            .map(|c| c.clone())
            .map_err(|e| ConfUtilError::MutexLockFailed(e.to_string()));
    }

    let file_path = WG_QUICKRS_CONFIG_FILE.get().unwrap();
    let config_str =
        fs::read_to_string(file_path).map_err(|e| ConfUtilError::Read(file_path.clone(), e))?;
    let config: Config = serde_yml::from_str(&config_str).map_err(ConfUtilError::Parse)?;
    let config_w_digest = ConfigWDigest::from_config_w_str(config.clone(), config_str)?;
    ConfigWDigest::set_or_init(config_w_digest.clone())?;
    log::info!("loaded config file");
    Ok(config_w_digest)
}

pub(crate) fn get_summary() -> Result<Summary, ConfUtilError> {
    let config_w_digest: ConfigWDigest = get_config_w_digest()?;
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
        agent: config_w_digest.config.agent,
        network: config_w_digest.config.network,
        telemetry,
        digest: config_w_digest.digest,
        status: status.value(),
        timestamp,
    })
}

pub(crate) fn set_config(config: &mut Config) -> Result<(), ConfUtilError> {
    config.network.updated_at = timestamp::get_now_timestamp_formatted();
    let config_str = serde_yml::to_string(&config).map_err(ConfUtilError::Serialization)?;
    let config_w_digest = ConfigWDigest::from_config_w_str(config.clone(), config_str.clone())?;
    ConfigWDigest::set_or_init(config_w_digest.clone())?;
    write_config(config_str)
}

pub(crate) fn write_config(config_str: String) -> Result<(), ConfUtilError> {
    let file_path = WG_QUICKRS_CONFIG_FILE.get().unwrap();
    let mut file =
        File::create(file_path).map_err(|e| ConfUtilError::Write(file_path.clone(), e))?;
    file.write_all(config_str.as_bytes())
        .map_err(|e| ConfUtilError::Write(file_path.clone(), e))?;

    log::info!("updated config file");
    Ok(())
}
