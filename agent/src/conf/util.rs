use crate::conf::timestamp;
use crate::wireguard::cmd::{show_dump, status_tunnel};
use crate::WG_RUSTEZE_CONFIG_FILE;
use config_wasm::types::{Config, Summary, WireGuardStatus};
use sha2::{Digest, Sha256};
use std::fs;
use std::fs::File;
use std::io::Write;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfUtilError {
    #[error("unable to write to (update) the config file: {0}")]
    FileWrite(String),
    #[error("unable to read the config file: {0}")]
    FileRead(String),
    #[error("unable to parse the config file: {0}")]
    InvalidConfigFile(String),
    #[error("unable to serialize the config object: {0}")]
    InvalidSerialize(String),
    #[error("unable to encode the digest: {0}")]
    DigestEncode(String),
}

pub(crate) fn get_config() -> Result<Config, ConfUtilError> {
    let file_contents = match fs::read_to_string(WG_RUSTEZE_CONFIG_FILE.get().unwrap()) {
        Ok(contents) => contents,
        Err(e) => { return Err(ConfUtilError::FileRead(e.to_string())); }
    };
    let mut config: Config = match serde_yml::from_str(&file_contents) {
        Ok(c) => c,
        Err(e) => { return Err(ConfUtilError::InvalidConfigFile(e.to_string())); }
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
    let file_contents = match fs::read_to_string(WG_RUSTEZE_CONFIG_FILE.get().unwrap()) {
        Ok(contents) => contents,
        Err(e) => { return Err(ConfUtilError::FileRead(e.to_string())); }
    };
    let digest = match base16ct::lower::encode_str(&Sha256::digest(file_contents.as_bytes()), &mut buf) {
        Ok(digest) => digest.to_string(),
        Err(e) => { return Err(ConfUtilError::DigestEncode(e.to_string())); }
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
        Err(e) => { return Err(ConfUtilError::InvalidSerialize(e.to_string())); }
    };

    let mut file = match File::create(WG_RUSTEZE_CONFIG_FILE.get().unwrap()) {
        Ok(f) => f,
        Err(e) => { return Err(ConfUtilError::FileWrite(e.to_string())); }
    };

    match file.write_all(config_str.as_bytes()) {
        Ok(_) => {},
        Err(e) => { return Err(ConfUtilError::FileWrite(e.to_string())); }
    };

    log::info!("updated config file");
    Ok(())
}
