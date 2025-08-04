use crate::WG_RUSTEZE_CONFIG_FILE;
use crate::conf::timestamp;
use crate::wireguard::cmd::{show_dump, status_tunnel};
use config_wasm::types::{Config, Summary, WireGuardStatus};
use sha2::{Digest, Sha256};
use std::fs;
use std::fs::File;
use std::io::Write;

pub(crate) fn get_config() -> Config {
    let file_contents = fs::read_to_string(
        WG_RUSTEZE_CONFIG_FILE
            .get()
            .expect("WG_RUSTEZE_CONFIG_FILE not set"),
    )
    .expect("Unable to open file");
    let mut config: Config = serde_yml::from_str(&file_contents).unwrap();

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
        set_config(&mut config);
    }

    config
}

pub(crate) fn get_summary() -> Summary {
    let config: Config = get_config();

    let mut buf = [0u8; 64];
    let file_contents = fs::read_to_string(
        WG_RUSTEZE_CONFIG_FILE
            .get()
            .expect("WG_RUSTEZE_CONFIG_FILE not set"),
    )
    .expect("Unable to open file");
    let digest = base16ct::lower::encode_str(&Sha256::digest(file_contents.as_bytes()), &mut buf)
        .expect("Unable to calculate network digest")
        .to_string();
    let status = match status_tunnel() {
        Ok(status) => status.value(),
        Err(e) => {
            log::error!("{e}");
            WireGuardStatus::UNKNOWN.value()
        }
    };
    let mut telemetry = Default::default();
    if status == WireGuardStatus::UP.value() {
        telemetry = show_dump(&config).unwrap_or_else(|e| {
            log::error!("{e}");
            Default::default()
        });
    }
    let timestamp = timestamp::get_now_timestamp_formatted();

    Summary {
        agent: config.agent,
        network: config.network,
        telemetry,
        digest,
        status,
        timestamp,
    }
}

pub(crate) fn set_config(config: &mut Config) {
    config.network.updated_at = timestamp::get_now_timestamp_formatted();
    let config_str = serde_yml::to_string(&config).expect("Failed to serialize config");

    let mut file = File::create(
        WG_RUSTEZE_CONFIG_FILE
            .get()
            .expect("WG_RUSTEZE_CONFIG_FILE not set"),
    )
    .expect("Failed to open config file");
    file.write_all(config_str.as_bytes())
        .expect("Failed to write to config file");
    log::info!("updated config file")
}
