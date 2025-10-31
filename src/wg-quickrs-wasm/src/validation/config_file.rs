#![cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use thiserror::Error;
use crate::helpers::remove_expired_reservations;
use crate::types::config::ConfigFile;
use crate::validation::error::*;
use crate::validation::agent::*;
use crate::validation::network::*;

#[derive(Error, Debug)]
pub enum ConfigFileValidationError {
    #[error("{0}: {1}")]
    Validation(String, ValidationError),
}

pub fn validate_config_file(config_file: &mut ConfigFile, config_file_path: &PathBuf) -> Result<(), ConfigFileValidationError> {
    // Validate Agent
    if config_file.agent.web.https.enabled {
        validate_tls_file(config_file_path, &config_file.agent.web.https.tls_cert).map_err(|e| {
            ConfigFileValidationError::Validation("agent.web.https.tls_cert".to_string(), e)
        })?;
        validate_tls_file(config_file_path, &config_file.agent.web.https.tls_key).map_err(|e| {
            ConfigFileValidationError::Validation("agent.web.https.tls_key".to_string(), e)
        })?;
    }
    if config_file.agent.firewall.enabled {
        validate_fw_utility(&config_file.agent.firewall.utility).map_err(|e| {
            ConfigFileValidationError::Validation("agent.firewall.utility".to_string(), e)
        })?;
        parse_and_validate_fw_gateway(&config_file.agent.firewall.gateway).map_err(|e| {
            ConfigFileValidationError::Validation("agent.firewall.gateway".to_string(), e)
        })?;
    }

    // Validate Network
    parse_and_validate_network_name(&config_file.network.name).map_err(|e| {
        ConfigFileValidationError::Validation("network.name".to_string(), e)
    })?;
    // skip network.subnet because if it can be deserialized, it means it's valid
    // skip network.this_peer because if it can be deserialized, it means it's valid

    // Validate peers
    for (peer_id, peer) in &config_file.network.peers {
        let peer_path = format!("network.peers.{}", peer_id);

        let mut temp_network = config_file.network.clone();
        temp_network.peers.remove(peer_id);

        // skip network.peers.{peer_id} because if it can be deserialized, it means it's valid
        parse_and_validate_peer_name(&peer.name).map_err(|e| {
            ConfigFileValidationError::Validation(format!("{}.name", peer_path), e)
        })?;
        validate_peer_address(&peer.address, &config_file.network).map_err(|e| {
            ConfigFileValidationError::Validation(format!("{}.address", peer_path), e)
        })?;
        // skip network.peers.{peer_id}.endpoint because if it can be deserialized, it means it's valid
        parse_and_validate_peer_kind(&peer.kind).map_err(|e| {
            ConfigFileValidationError::Validation(format!("{}.kind", peer_path), e)
        })?;
        if peer.icon.enabled {
            parse_and_validate_peer_icon_src(&peer.icon.src).map_err(|e| {
                ConfigFileValidationError::Validation(format!("{}.icon", peer_path), e)
            })?;
        }
        // skip network.peers.{peer_id}.dns because if it can be deserialized, it means it's valid
        if peer.mtu.enabled {
            validate_peer_mtu_value(peer.mtu.value).map_err(|e| {
                ConfigFileValidationError::Validation(format!("{}.mtu", peer_path), e)
            })?;
        }
        // skip network.peers.{peer_id}.private_key because if it can be deserialized, it means it's valid

        for (script_type, scripts) in peer.scripts.clone() {
            for (i, script) in scripts.into_iter().enumerate() {
                if script.enabled {
                    parse_and_validate_peer_script(&script.script).map_err(|e| {
                        ConfigFileValidationError::Validation(format!("{peer_path}.scripts.{script_type}.{i}"), e)
                    })?;
                }
            }
        }
    }

    // Validate connections
    // skip network.connections.{conn_id} because if it can be deserialized, it means it's valid
    // skip network.connections.{conn_id}.pre_shared_key because if it can be deserialized, it means it's valid
    // skip network.connections.{conn_id}.allowed_ips_a_to_b because if it can be deserialized, it means it's valid
    // skip network.connections.{conn_id}.allowed_ips_b_to_a because if it can be deserialized, it means it's valid
    // skip network.connections.{conn_id}.persistent_keepalive because if it can be deserialized, it means it's valid

    // Validate defaults
    let defaults_path = "network.defaults";
    // skip network.defaults.peer.endpoint because if it can be deserialized, it means it's valid
    parse_and_validate_peer_kind(&config_file.network.defaults.peer.kind).map_err(|e| {
        ConfigFileValidationError::Validation(format!("{}.peer.kind", defaults_path), e)
    })?;
    if config_file.network.defaults.peer.icon.enabled {
        parse_and_validate_peer_icon_src(&config_file.network.defaults.peer.icon.src).map_err(|e| {
            ConfigFileValidationError::Validation(format!("{}.peer.icon", defaults_path), e)
        })?;
    }
    // skip network.defaults.peer.dns because if it can be deserialized, it means it's valid
    if config_file.network.defaults.peer.mtu.enabled {
        validate_peer_mtu_value(config_file.network.defaults.peer.mtu.value).map_err(|e| {
            ConfigFileValidationError::Validation(format!("{}.peer.mtu", defaults_path), e)
        })?;
    }
    for (script_type, scripts) in config_file.network.defaults.peer.scripts.clone() {
        for (i, script) in scripts.into_iter().enumerate() {
            if script.enabled {
                parse_and_validate_peer_script(&script.script).map_err(|e| {
                    ConfigFileValidationError::Validation(format!("{defaults_path}.scripts.{script_type}.{i}"), e)
                })?;
            }
        }
    }

    // skip network.defaults.connection.persistent_keepalive because if it can be deserialized, it means it's valid

    // Validate reservations
    for (address, _reservation) in &config_file.network.reservations {
        let mut temp_network = config_file.network.clone();
        temp_network.reservations.remove(address);

        validate_peer_address(&address, &temp_network).map_err(|e| {
            ConfigFileValidationError::Validation(format!("network.reservations.{{{address}}}"), e)
        })?;
        // skip network.reservations.{address}.peer_id because if it can be deserialized, it means it's valid
        // skip network.reservations.{address}.valid_until because if it can be deserialized, it means it's valid
    }
    remove_expired_reservations(&mut config_file.network);

    // skip network.updated_at because if it can be deserialized, it means it's valid

    Ok(())
}
