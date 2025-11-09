// ============================================================================
// Macros for reducing boilerplate
// ============================================================================

use crate::conf;
use crate::commands::config::{parse_connection_id, ConfigCommandError};
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;
use wg_quickrs_lib::validation::agent::{parse_and_validate_fw_gateway, validate_fw_utility, validate_tls_file};
use wg_quickrs_lib::validation::error::ValidationError;
use crate::WG_QUICKRS_CONFIG_FOLDER;

/// Macro for implementing port setter functions
macro_rules! impl_port_setter {
    ($fn_name:ident, $($field:ident).+, $port_name:expr) => {
        pub fn $fn_name(port: u16) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            config.$($field).+.port = port;
            log::info!("Setting {} port to {}", $port_name, port);
            conf::util::set_config(&mut config)?;
            Ok(())
        }
    };
}

/// Macro for implementing generic setter functions
macro_rules! impl_setter {
    // Simple setter with Display
    ($fn_name:ident, $value_type:ty, $($field:ident).+, $field_name:expr) => {
        pub fn $fn_name(value: &$value_type) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            log::info!("Setting {} to {}", $field_name, value);
            config.$($field).+ = value.clone();
            conf::util::set_config(&mut config)?;
            Ok(())
        }
    };
    // Setter with custom display format
    ($fn_name:ident, $value_type:ty, $($field:ident).+, $field_name:expr, display: $display_fn:expr) => {
        pub fn $fn_name(value: &$value_type) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            log::info!("Setting {} to {}", $field_name, $display_fn(value));
            config.$($field).+ = value.clone();
            conf::util::set_config(&mut config)?;
            Ok(())
        }
    };
    // Setter with transformation/validation
    ($fn_name:ident, $value_type:ty, $($field:ident).+, $field_name:expr, transform: $transformer:expr) => {
        pub fn $fn_name(value: &$value_type) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            log::info!("Setting {} to {}", $field_name, value);
            config.$($field).+ = $transformer(value)?;
            conf::util::set_config(&mut config)?;
            Ok(())
        }
    };
    // Setter with transformation and custom display
    ($fn_name:ident, $value_type:ty, $($field:ident).+, $field_name:expr, display: $display_fn:expr, transform: $transformer:expr) => {
        pub fn $fn_name(value: &$value_type) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            log::info!("Setting {} to {}", $field_name, $display_fn(value));
            config.$($field).+ = $transformer(value)?;
            conf::util::set_config(&mut config)?;
            Ok(())
        }
    };
}


// ============================================================================
// Agent Web Configuration Functions
// ============================================================================

impl_setter!(set_agent_web_address, Ipv4Addr, agent.web.address, "agent address");


impl_port_setter!(set_agent_web_http_port, agent.web.http, "HTTP");

impl_setter!(
    set_agent_web_http_tls_cert,
    PathBuf,
    agent.web.https.tls_cert,
    "TLS certificate",
    display: |p: &PathBuf| format!("{}", p.display()),
    transform: |tls_cert: &PathBuf| {
        let wg_quickrs_conf_folder = WG_QUICKRS_CONFIG_FOLDER.get().unwrap();
        validate_tls_file(wg_quickrs_conf_folder, tls_cert)
    }
);

impl_setter!(
    set_agent_web_http_tls_key,
    PathBuf,
    agent.web.https.tls_key,
    "TLS key",
    display: |p: &PathBuf| format!("{}", p.display()),
    transform: |tls_key: &PathBuf| {
        let wg_quickrs_conf_folder = WG_QUICKRS_CONFIG_FOLDER.get().unwrap();
        validate_tls_file(wg_quickrs_conf_folder, tls_key)
    }
);


impl_port_setter!(set_agent_web_https_port, agent.web.https, "HTTPS");

// ============================================================================
// Agent VPN Configuration Functions
// ============================================================================


impl_port_setter!(set_agent_vpn_port, agent.vpn, "VPN");

// ============================================================================
// Agent Firewall Configuration Functions
// ============================================================================


impl_setter!(
    set_agent_firewall_utility,
    PathBuf,
    agent.firewall.utility,
    "firewall utility",
    display: |p: &PathBuf| format!("{}", p.display()),
    transform: |utility: &PathBuf| validate_fw_utility(utility)
);

impl_setter!(
    set_agent_firewall_gateway,
    str,
    agent.firewall.gateway,
    "firewall gateway",
    transform: |gateway: &str| parse_and_validate_fw_gateway(gateway)
);

/// Set network name
pub fn set_network_name(name: String) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    config.network.name = name.clone();
    log::info!("Set network name to: {}", name);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Set network subnet
pub fn set_network_subnet(subnet_str: &str) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let subnet = ipnet::Ipv4Net::from_str(subnet_str)
        .map_err(|_| ConfigCommandError::Validation(ValidationError::NotCIDR()))?;
    config.network.subnet = subnet;
    log::info!("Set network subnet to: {}", subnet);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Set peer name
pub fn set_peer_name(id: &Uuid, name: String) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let peer = config.network.peers.get_mut(id).ok_or(ConfigCommandError::PeerNotFound(*id))?;
    peer.name = name.clone();
    log::info!("Set peer {} name to: {}", id, name);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Set peer address
pub fn set_peer_address(id: &Uuid, address: Ipv4Addr) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let peer = config.network.peers.get_mut(id).ok_or(ConfigCommandError::PeerNotFound(*id))?;
    peer.address = address;
    log::info!("Set peer {} address to: {}", id, address);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Set peer endpoint
pub fn set_peer_endpoint(id: &Uuid, endpoint_str: &str) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let peer = config.network.peers.get_mut(id).ok_or(ConfigCommandError::PeerNotFound(*id))?;
    let endpoint_address = wg_quickrs_lib::validation::network::parse_and_validate_peer_endpoint(endpoint_str)?;
    peer.endpoint.address = endpoint_address;
    peer.endpoint.enabled = true;
    log::info!("Set peer {} endpoint to: {}", id, endpoint_str);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Set peer kind
pub fn set_peer_kind(id: &Uuid, kind: &str) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let peer = config.network.peers.get_mut(id).ok_or(ConfigCommandError::PeerNotFound(*id))?;
    peer.kind = kind.to_string();
    log::info!("Set peer {} kind to: {}", id, kind);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Set peer icon source
pub fn set_peer_icon(id: &Uuid, src: &str) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let peer = config.network.peers.get_mut(id).ok_or(ConfigCommandError::PeerNotFound(*id))?;
    peer.icon.src = src.to_string();
    peer.icon.enabled = true;
    log::info!("Set peer {} icon to: {}", id, src);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Set peer DNS addresses
pub fn set_peer_dns(id: &Uuid, addresses_str: &str) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let peer = config.network.peers.get_mut(id).ok_or(ConfigCommandError::PeerNotFound(*id))?;

    let addresses: Vec<Ipv4Addr> = addresses_str.split(',')
        .map(|s| s.trim().parse::<Ipv4Addr>()
            .map_err(|_| ConfigCommandError::Validation(ValidationError::NotIPv4Address())))
        .collect::<Result<Vec<_>, _>>()?;

    peer.dns.addresses = addresses;
    peer.dns.enabled = true;
    log::info!("Set peer {} DNS to: {}", id, addresses_str);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Set peer MTU value
pub fn set_peer_mtu(id: &Uuid, value: u16) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let peer = config.network.peers.get_mut(id).ok_or(ConfigCommandError::PeerNotFound(*id))?;
    peer.mtu.value = value;
    peer.mtu.enabled = true;
    log::info!("Set peer {} MTU to: {}", id, value);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Set connection allowed IPs A to B
pub fn set_connection_allowed_ips_a_to_b(id_str: &str, ips_str: &str) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let conn_id = parse_connection_id(id_str)?;
    let connection = config.network.connections.get_mut(&conn_id)
        .ok_or_else(|| ConfigCommandError::ConnectionNotFound(id_str.to_string()))?;

    let ips: Vec<ipnet::Ipv4Net> = ips_str.split(',')
        .map(|s| ipnet::Ipv4Net::from_str(s.trim())
            .map_err(|_| ConfigCommandError::Validation(ValidationError::NotCIDR())))
        .collect::<Result<Vec<_>, _>>()?;

    connection.allowed_ips_a_to_b = ips;
    log::info!("Set connection {} allowed IPs A->B to: {}", id_str, ips_str);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Set connection allowed IPs B to A
pub fn set_connection_allowed_ips_b_to_a(id_str: &str, ips_str: &str) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let conn_id = parse_connection_id(id_str)?;
    let connection = config.network.connections.get_mut(&conn_id)
        .ok_or_else(|| ConfigCommandError::ConnectionNotFound(id_str.to_string()))?;

    let ips: Vec<ipnet::Ipv4Net> = ips_str.split(',')
        .map(|s| ipnet::Ipv4Net::from_str(s.trim())
            .map_err(|_| ConfigCommandError::Validation(ValidationError::NotCIDR())))
        .collect::<Result<Vec<_>, _>>()?;

    connection.allowed_ips_b_to_a = ips;
    log::info!("Set connection {} allowed IPs B->A to: {}", id_str, ips_str);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Set connection persistent keepalive
pub fn set_connection_persistent_keepalive(id_str: &str, period: u16) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let conn_id = parse_connection_id(id_str)?;
    let connection = config.network.connections.get_mut(&conn_id)
        .ok_or_else(|| ConfigCommandError::ConnectionNotFound(id_str.to_string()))?;
    connection.persistent_keepalive.period = period;
    connection.persistent_keepalive.enabled = true;
    log::info!("Set connection {} persistent keepalive to: {}", id_str, period);
    conf::util::set_config(&mut config)?;
    Ok(())
}

// ============================================================================
// Defaults Setter Functions - Set default configuration for peers/connections
// ============================================================================

/// Set default peer kind
pub fn set_defaults_peer_kind(kind: &str) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    config.network.defaults.peer.kind = kind.to_string();
    log::info!("Set default peer kind to: {}", kind);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Set default peer endpoint
pub fn set_defaults_peer_endpoint(endpoint_str: &str) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    let endpoint_address = wg_quickrs_lib::validation::network::parse_and_validate_peer_endpoint(endpoint_str)?;
    config.network.defaults.peer.endpoint.address = endpoint_address;
    config.network.defaults.peer.endpoint.enabled = true;
    log::info!("Set default peer endpoint to: {}", endpoint_str);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Set default peer icon source
pub fn set_defaults_peer_icon(src: &str) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    config.network.defaults.peer.icon.src = src.to_string();
    config.network.defaults.peer.icon.enabled = true;
    log::info!("Set default peer icon to: {}", src);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Set default peer DNS addresses
pub fn set_defaults_peer_dns(addresses_str: &str) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;

    let addresses: Vec<Ipv4Addr> = addresses_str.split(',')
        .map(|s| s.trim().parse::<Ipv4Addr>()
            .map_err(|_| ConfigCommandError::Validation(ValidationError::NotIPv4Address())))
        .collect::<Result<Vec<_>, _>>()?;

    config.network.defaults.peer.dns.addresses = addresses;
    config.network.defaults.peer.dns.enabled = true;
    log::info!("Set default peer DNS to: {}", addresses_str);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Set default peer MTU value
pub fn set_defaults_peer_mtu(value: u16) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    config.network.defaults.peer.mtu.value = value;
    config.network.defaults.peer.mtu.enabled = true;
    log::info!("Set default peer MTU to: {}", value);
    conf::util::set_config(&mut config)?;
    Ok(())
}

/// Set default connection persistent keepalive
pub fn set_defaults_connection_persistent_keepalive(period: u16) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    config.network.defaults.connection.persistent_keepalive.period = period;
    config.network.defaults.connection.persistent_keepalive.enabled = true;
    log::info!("Set default connection persistent keepalive to: {}", period);
    conf::util::set_config(&mut config)?;
    Ok(())
}

