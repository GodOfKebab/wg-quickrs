use crate::conf::timestamp;
use crate::wireguard::cmd::{show_dump, status_tunnel};
use crate::WG_RUSTEZE_CONFIG_FILE;
use config_wasm::types::{Config, Summary, WireGuardStatus};
use sha2::{Digest, Sha256};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use thiserror::Error;

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
}

pub(crate) fn get_config() -> Result<Config, ConfUtilError> {
    let file_path = WG_RUSTEZE_CONFIG_FILE.get().unwrap();
    let file_contents = match fs::read_to_string(file_path) {
        Ok(contents) => contents,
        Err(e) => {
            return Err(ConfUtilError::Read(file_path.clone(), e));
        }
    };
    let mut config: Config = match serde_yml::from_str(&file_contents) {
        Ok(c) => c,
        Err(e) => {
            return Err(ConfUtilError::Parse(e));
        }
    };

    // Make sure agent fields get precedence over network fields
    if config
        .network
        .peers
        .get(&config.network.this_peer)
        .unwrap()
        .endpoint
        .value
        != format!("{}:{}", config.agent.address, config.agent.vpn.port)
    {
        log::warn!(
            "detected mismatch between configured wg-rusteze agent endpoints and wireguard peer endpoints! overriding wireguard peer endpoints"
        );
        config
            .network
            .peers
            .get_mut(&config.network.this_peer)
            .unwrap()
            .endpoint
            .value = format!("{}:{}", config.agent.address, config.agent.vpn.port);
        set_config(&mut config)?;
    }

    Ok(config)
}

pub(crate) fn get_summary() -> Result<Summary, ConfUtilError> {
    let config: Config = get_config()?;

    let mut buf = [0u8; 64];
    let file_path = WG_RUSTEZE_CONFIG_FILE.get().unwrap();
    let file_contents = match fs::read_to_string(file_path) {
        Ok(contents) => contents,
        Err(e) => {
            return Err(ConfUtilError::Read(file_path.clone(), e));
        }
    };
    let digest =
        match base16ct::lower::encode_str(&Sha256::digest(file_contents.as_bytes()), &mut buf) {
            Ok(digest) => digest.to_string(),
            Err(e) => {
                return Err(ConfUtilError::DigestEncoding(e));
            }
        };
    let status = status_tunnel().unwrap_or_else(|e| {
        log::error!("{e}");
        WireGuardStatus::UNKNOWN
    });
    let mut telemetry = Default::default();
    if status.value() == WireGuardStatus::UP.value() {
        telemetry = show_dump(&config).unwrap_or_else(|e| {
            log::error!("{e}");
            Default::default()
        });
    }
    let timestamp = timestamp::get_now_timestamp_formatted();

    Ok(Summary {
        agent: config.agent,
        network: config.network,
        telemetry,
        digest,
        status: status.value(),
        timestamp,
    })
}

pub(crate) fn set_config(config: &mut Config) -> Result<(), ConfUtilError> {
    config.network.updated_at = timestamp::get_now_timestamp_formatted();
    let config_str = match serde_yml::to_string(&config) {
        Ok(s) => s,
        Err(e) => {
            return Err(ConfUtilError::Serialization(e));
        }
    };

    let file_path = WG_RUSTEZE_CONFIG_FILE.get().unwrap();
    let mut file = match File::create(file_path) {
        Ok(f) => f,
        Err(e) => {
            return Err(ConfUtilError::Write(file_path.clone(), e));
        }
    };

    match file.write_all(config_str.as_bytes()) {
        Ok(_) => {}
        Err(e) => {
            return Err(ConfUtilError::Write(file_path.clone(), e));
        }
    };

    log::info!("updated config file");
    Ok(())
}
