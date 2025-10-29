use crate::{WG_QUICKRS_CONFIG_FOLDER, WG_QUICKRS_CONFIG_FILE};
use wg_quickrs_wasm::validation::{check_field_enabled_value, check_field_str, check_internal_address, is_cidr, CheckResult};
use std::path::Path;
use wg_quickrs_wasm::types::EnabledValue;
use uuid::Uuid;
use crate::conf::util::{ConfUtilError, ConfigFile};

// Macro to check a validation result and return an error if failed
macro_rules! validate_field {
    ($validation_func:expr, $field_path:expr) => {
        {
            let result = $validation_func;
            if !result.status {
                return Err(ConfUtilError::Validation(WG_QUICKRS_CONFIG_FILE.get().unwrap().clone(), format!("{}: {}", $field_path, result.msg)));
            }
        }
    };
}

// Macro to validate scripts for a given peer/context
macro_rules! validate_scripts {
    ($scripts:expr, $context_path:expr) => {
        for script in &$scripts.pre_up {
            validate_field!(
                check_field_enabled_value_agent("pre_up", script),
                format!("{}.scripts.pre_up", $context_path)
            );
        }
        for script in &$scripts.post_up {
            validate_field!(
                check_field_enabled_value_agent("post_up", script),
                format!("{}.scripts.post_up", $context_path)
            );
        }
        for script in &$scripts.pre_down {
            validate_field!(
                check_field_enabled_value_agent("pre_down", script),
                format!("{}.scripts.pre_down", $context_path)
            );
        }
        for script in &$scripts.post_down {
            validate_field!(
                check_field_enabled_value_agent("post_down", script),
                format!("{}.scripts.post_down", $context_path)
            );
        }
    };
}


pub fn validate_config_file(config_file: &ConfigFile) -> Result<(), ConfUtilError> {
    // Validate Agent fields
    validate_field!(check_field_str("generic-address", &config_file.agent.web.address), "agent.web.address");
    if config_file.agent.web.https.enabled {
        validate_field!(check_field_path_agent("path", &config_file.agent.web.https.tls_cert), "agent.web.https.tls_cert");
        validate_field!(check_field_path_agent("path", &config_file.agent.web.https.tls_key), "agent.web.https.tls_key");
    }
    if config_file.agent.firewall.enabled {
        validate_field!(check_field_path_agent("firewall-utility", &config_file.agent.firewall.utility), "agent.firewall.utility");
        validate_field!(check_field_str_agent("firewall-gateway", &config_file.agent.firewall.gateway), "agent.firewall.gateway");
    }

    // Validate Network fields
    validate_field!(check_field_str_agent("identifier", &config_file.network.identifier), "network.identifier");
    validate_field!(check_field_str_agent("subnet", &config_file.network.subnet), "network.subnet");
    validate_field!(check_field_str_agent("peer_id", &config_file.network.this_peer), "network.this_peer");

    // Validate peers
    for (peer_id, peer) in &config_file.network.peers {
        let peer_path = format!("network.peers.{}", peer_id);

        let mut temp_network = config_file.network.clone();
        temp_network.peers.remove(peer_id);
        
        validate_field!(check_field_str_agent("peer_id", peer_id), peer_path);
        validate_field!(check_field_str("name", &peer.name), format!("{}.name", peer_path));
        validate_field!(check_internal_address(&peer.address, &temp_network), format!("{}.address", peer_path));
        validate_field!(check_field_enabled_value("endpoint", &peer.endpoint), format!("{}.endpoint", peer_path));
        validate_field!(check_field_str("kind", &peer.kind), format!("{}.kind", peer_path));
        validate_field!(check_field_enabled_value("icon", &peer.icon), format!("{}.icon", peer_path));
        validate_field!(check_field_enabled_value("dns", &peer.dns), format!("{}.dns", peer_path));
        validate_field!(check_field_enabled_value("mtu", &peer.mtu), format!("{}.mtu", peer_path));
        validate_field!(check_field_str("private_key", &peer.private_key), format!("{}.private_key", peer_path));

        // Validate scripts using macro
        validate_scripts!(peer.scripts, peer_path);
    }

    // Validate connections
    for (connection_id, connection) in &config_file.network.connections {
        let conn_path = format!("network.connections.{}", connection_id);
        
        validate_field!(
            check_field_str("pre_shared_key", &connection.pre_shared_key),
            format!("{}.pre_shared_key", conn_path)
        );
        validate_field!(
            check_field_str("allowed_ips_a_to_b", &connection.allowed_ips_a_to_b),
            format!("{}.allowed_ips_a_to_b", conn_path)
        );
        validate_field!(
            check_field_str("allowed_ips_b_to_a", &connection.allowed_ips_b_to_a),
            format!("{}.allowed_ips_b_to_a", conn_path)
        );
        validate_field!(
            check_field_enabled_value("persistent_keepalive", &connection.persistent_keepalive),
            format!("{}.persistent_keepalive", conn_path)
        );
    }

    // Validate defaults
    let defaults_path = "network.defaults";
    validate_field!(
        check_field_enabled_value("endpoint", &config_file.network.defaults.peer.endpoint),
        format!("{}.peer.endpoint", defaults_path)
    );
    validate_field!(
        check_field_str("kind", &config_file.network.defaults.peer.kind),
        format!("{}.peer.kind", defaults_path)
    );
    validate_field!(
        check_field_enabled_value("icon", &config_file.network.defaults.peer.icon),
        format!("{}.peer.icon", defaults_path)
    );
    validate_field!(
        check_field_enabled_value("dns", &config_file.network.defaults.peer.dns),
        format!("{}.peer.dns", defaults_path)
    );
    validate_field!(
        check_field_enabled_value("mtu", &config_file.network.defaults.peer.mtu),
        format!("{}.peer.mtu", defaults_path)
    );
    validate_field!(
        check_field_enabled_value("persistent_keepalive", &config_file.network.defaults.connection.persistent_keepalive),
        format!("{}.connection.persistent_keepalive", defaults_path)
    );

    // Validate default scripts using macro
    validate_scripts!(config_file.network.defaults.peer.scripts, "network.defaults.peer");

    // validate reservations
    for (address, reservation) in &config_file.network.reservations {
        let reservations_path = format!("network.reservations.{}", address);
        let mut temp_network = config_file.network.clone();
        temp_network.reservations.remove(address);

        validate_field!(check_internal_address(address, &temp_network), reservations_path);
        validate_field!(check_field_str_agent("peer_id", &reservation.peer_id), format!("{}.peer_id", reservations_path));
    }

    Ok(())
}
