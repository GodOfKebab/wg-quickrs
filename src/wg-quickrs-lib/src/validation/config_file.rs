#![cfg(not(target_arch = "wasm32"))]
use std::path::Path;
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

pub fn validate_config_file(config_file: &mut ConfigFile, config_folder_path: &Path) -> Result<(), ConfigFileValidationError> {
    // Validate Agent
    if config_file.agent.web.https.enabled {
        validate_tls_file(config_folder_path, &config_file.agent.web.https.tls_cert).map_err(|e| {
            ConfigFileValidationError::Validation("agent.web.https.tls_cert".to_string(), e)
        })?;
        validate_tls_file(config_folder_path, &config_file.agent.web.https.tls_key).map_err(|e| {
            ConfigFileValidationError::Validation("agent.web.https.tls_key".to_string(), e)
        })?;
    }

    // Validate VPN settings
    if config_file.agent.vpn.enabled {
        validate_wg_tool(&config_file.agent.vpn.wg).map_err(|e| {
            ConfigFileValidationError::Validation("agent.vpn.wg".to_string(), e)
        })?;

        if config_file.agent.vpn.wg_userspace.enabled {
            validate_wg_userspace_binary(&config_file.agent.vpn.wg_userspace.binary).map_err(|e| {
                ConfigFileValidationError::Validation("agent.vpn.wg_userspace.binary".to_string(), e)
            })?;
        }
    }
    // Validate Firewall scripts
    for (protocol, scripts_map) in [
        ("http", &config_file.agent.firewall.http),
        ("https", &config_file.agent.firewall.https),
    ] {
        for (script_type, scripts) in scripts_map.clone() {
            for (i, script) in scripts.iter().enumerate() {
                if script.enabled {
                    parse_and_validate_peer_script(&script.script).map_err(|e| {
                        ConfigFileValidationError::Validation(
                            format!("agent.firewall.{protocol}.{script_type}.{i}"),
                            e,
                        )
                    })?;
                }
            }
        }
    }
    for (script_type, scripts) in config_file.agent.firewall.vpn.clone() {
        for (i, script) in scripts.iter().enumerate() {
            if script.enabled {
                parse_and_validate_peer_script(&script.script).map_err(|e| {
                    ConfigFileValidationError::Validation(
                        format!("agent.firewall.vpn.{script_type}.{i}"),
                        e,
                    )
                })?;
            }
        }
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
        validate_peer_address(&peer.address, &temp_network).map_err(|e| {
            ConfigFileValidationError::Validation(format!("{}.address", peer_path), e)
        })?;
        validate_peer_endpoint(&peer.endpoint).map_err(|e| {
            ConfigFileValidationError::Validation(format!("{}.endpoint", peer_path), e)
        })?;
        parse_and_validate_peer_kind(&peer.kind).map_err(|e| {
            ConfigFileValidationError::Validation(format!("{}.kind", peer_path), e)
        })?;
        validate_peer_icon(&peer.icon).map_err(|e| {
            ConfigFileValidationError::Validation(format!("{}.icon", peer_path), e)
        })?;
        validate_peer_dns(&peer.dns).map_err(|e| {
            ConfigFileValidationError::Validation(format!("{}.dns", peer_path), e)
        })?;
        validate_peer_mtu(&peer.mtu).map_err(|e| {
            ConfigFileValidationError::Validation(format!("{}.mtu", peer_path), e)
        })?;
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
    for (connection_id, connection) in &config_file.network.connections {
        let conn_path = format!("network.connections.{}", connection_id);
        // skip network.connections.{conn_id} because if it can be deserialized, it means it's valid
        // skip network.connections.{conn_id}.pre_shared_key because if it can be deserialized, it means it's valid
        // skip network.connections.{conn_id}.allowed_ips_a_to_b because if it can be deserialized, it means it's valid
        // skip network.connections.{conn_id}.allowed_ips_b_to_a because if it can be deserialized, it means it's valid
        validate_conn_persistent_keepalive(&connection.persistent_keepalive).map_err(|e| {
            ConfigFileValidationError::Validation(format!("{}.persistent_keepalive", conn_path), e)
        })?;
    }

    // Validate defaults
    let defaults_path = "network.defaults";
    parse_and_validate_peer_kind(&config_file.network.defaults.peer.kind).map_err(|e| {
        ConfigFileValidationError::Validation(format!("{}.peer.kind", defaults_path), e)
    })?;
    validate_peer_icon(&config_file.network.defaults.peer.icon).map_err(|e| {
        ConfigFileValidationError::Validation(format!("{}.peer.icon", defaults_path), e)
    })?;
    validate_peer_dns(&config_file.network.defaults.peer.dns).map_err(|e| {
        ConfigFileValidationError::Validation(format!("{}.peer.dns", defaults_path), e)
    })?;
    validate_peer_mtu(&config_file.network.defaults.peer.mtu).map_err(|e| {
        ConfigFileValidationError::Validation(format!("{}.peer.mtu", defaults_path), e)
    })?;
    for (script_type, scripts) in config_file.network.defaults.peer.scripts.clone() {
        validate_peer_scripts(&scripts).map_err(|e| {
            ConfigFileValidationError::Validation(format!("{defaults_path}.peer.scripts.{script_type}"), e)
        })?;
    }

    validate_conn_persistent_keepalive(&config_file.network.defaults.connection.persistent_keepalive).map_err(|e| {
        ConfigFileValidationError::Validation(format!("{}.connection.persistent_keepalive", defaults_path), e)
    })?;

    // Validate reservations
    for address in config_file.network.reservations.keys() {
        let mut temp_network = config_file.network.clone();
        temp_network.reservations.remove(address);

        validate_peer_address(address, &temp_network).map_err(|e| {
            ConfigFileValidationError::Validation(format!("network.reservations.{{{address}}}"), e)
        })?;
        // skip network.reservations.{address}.peer_id because if it can be deserialized, it means it's valid
        // skip network.reservations.{address}.valid_until because if it can be deserialized, it means it's valid
    }
    remove_expired_reservations(&mut config_file.network);

    // skip network.updated_at because if it can be deserialized, it means it's valid

    Ok(())
}
