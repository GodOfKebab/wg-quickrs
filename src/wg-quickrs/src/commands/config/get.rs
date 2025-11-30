use crate::commands::config::parse_connection_id;
use crate::commands::config::Ipv4Addr;
use crate::commands::config::ConfigCommandError;
use crate::commands::config::Uuid;
use crate::commands::config::print_as_yaml;
use crate::conf;

/// Macro for implementing indexed peer getters
macro_rules! impl_peer_getter {
    // Get entire peer struct as YAML
    ($fn_name:ident) => {
        pub fn $fn_name(id: &Uuid) -> Result<(), ConfigCommandError> {
            let config = conf::util::get_config()?;
            let peer = config.network.peers.get(id)
                .ok_or_else(|| ConfigCommandError::PeerNotFound(*id))?;
            print_as_yaml(peer)
        }
    };
    // Get peer field as YAML
    ($fn_name:ident, $($field:ident).+, yaml) => {
        pub fn $fn_name(id: &Uuid) -> Result<(), ConfigCommandError> {
            let config = conf::util::get_config()?;
            let peer = config.network.peers.get(id)
                .ok_or_else(|| ConfigCommandError::PeerNotFound(*id))?;
            print_as_yaml(&peer.$($field).+)
        }
    };
    // Get peer field as plain value
    ($fn_name:ident, $($field:ident).+) => {
        pub fn $fn_name(id: &Uuid) -> Result<(), ConfigCommandError> {
            let config = conf::util::get_config()?;
            let peer = config.network.peers.get(id)
                .ok_or_else(|| ConfigCommandError::PeerNotFound(*id))?;
            println!("{}", peer.$($field).+);
            Ok(())
        }
    };
}

/// Macro for implementing indexed connection getters
macro_rules! impl_connection_getter {
    // Get entire connection struct as YAML
    ($fn_name:ident) => {
        pub fn $fn_name(id_str: &str) -> Result<(), ConfigCommandError> {
            let config = conf::util::get_config()?;
            let conn_id = parse_connection_id(id_str)?;
            let connection = config.network.connections.get(&conn_id)
                .ok_or_else(|| ConfigCommandError::ConnectionNotFound(id_str.to_string()))?;
            print_as_yaml(connection)
        }
    };
    // Get connection field as YAML
    ($fn_name:ident, $($field:ident).+, yaml) => {
        pub fn $fn_name(id_str: &str) -> Result<(), ConfigCommandError> {
            let config = conf::util::get_config()?;
            let conn_id = parse_connection_id(id_str)?;
            let connection = config.network.connections.get(&conn_id)
                .ok_or_else(|| ConfigCommandError::ConnectionNotFound(id_str.to_string()))?;
            print_as_yaml(&connection.$($field).+)
        }
    };
    // Get connection field as plain value
    ($fn_name:ident, $($field:ident).+) => {
        pub fn $fn_name(id_str: &str) -> Result<(), ConfigCommandError> {
            let config = conf::util::get_config()?;
            let conn_id = parse_connection_id(id_str)?;
            let connection = config.network.connections.get(&conn_id)
                .ok_or_else(|| ConfigCommandError::ConnectionNotFound(id_str.to_string()))?;
            println!("{}", connection.$($field).+);
            Ok(())
        }
    };
}

/// Macro for implementing indexed reservation getters
macro_rules! impl_reservation_getter {
    // Get entire reservation struct as YAML
    ($fn_name:ident) => {
        pub fn $fn_name(ip: &Ipv4Addr) -> Result<(), ConfigCommandError> {
            let config = conf::util::get_config()?;
            let reservation = config.network.reservations.get(ip)
                .ok_or_else(|| ConfigCommandError::ReservationNotFound(*ip))?;
            print_as_yaml(reservation)
        }
    };
    // Get reservation field as plain value
    ($fn_name:ident, $($field:ident).+) => {
        pub fn $fn_name(ip: &Ipv4Addr) -> Result<(), ConfigCommandError> {
            let config = conf::util::get_config()?;
            let reservation = config.network.reservations.get(ip)
                .ok_or_else(|| ConfigCommandError::ReservationNotFound(*ip))?;
            println!("{}", reservation.$($field).+);
            Ok(())
        }
    };
}

// Network indexed getters for peers (using macros)
impl_peer_getter!(get_network_peer);
impl_peer_getter!(get_network_peer_name, name);
impl_peer_getter!(get_network_peer_address, address);
impl_peer_getter!(get_network_peer_endpoint, endpoint, yaml);
impl_peer_getter!(get_network_peer_endpoint_enabled, endpoint.enabled);
impl_peer_getter!(get_network_peer_endpoint_address, endpoint.address, yaml);
impl_peer_getter!(get_network_peer_kind, kind);
impl_peer_getter!(get_network_peer_icon, icon, yaml);
impl_peer_getter!(get_network_peer_icon_enabled, icon.enabled);
impl_peer_getter!(get_network_peer_icon_src, icon.src);
impl_peer_getter!(get_network_peer_dns, dns, yaml);
impl_peer_getter!(get_network_peer_dns_enabled, dns.enabled);
impl_peer_getter!(get_network_peer_dns_addresses, dns.addresses, yaml);
impl_peer_getter!(get_network_peer_mtu, mtu, yaml);
impl_peer_getter!(get_network_peer_mtu_enabled, mtu.enabled);
impl_peer_getter!(get_network_peer_mtu_value, mtu.value);
impl_peer_getter!(get_network_peer_scripts, scripts, yaml);
impl_peer_getter!(get_network_peer_private_key, private_key);
impl_peer_getter!(get_network_peer_amnezia_parameters, amnezia_parameters, yaml);
impl_peer_getter!(get_network_peer_amnezia_parameters_jc, amnezia_parameters.jc);
impl_peer_getter!(get_network_peer_amnezia_parameters_jmin, amnezia_parameters.jmin);
impl_peer_getter!(get_network_peer_amnezia_parameters_jmax, amnezia_parameters.jmax);
impl_peer_getter!(get_network_peer_created_at, created_at);
impl_peer_getter!(get_network_peer_updated_at, updated_at);

// Network indexed getters for connections (using macros)

impl_connection_getter!(get_network_connection);
impl_connection_getter!(get_network_connection_enabled, enabled);
impl_connection_getter!(get_network_connection_pre_shared_key, pre_shared_key);
impl_connection_getter!(get_network_connection_persistent_keepalive, persistent_keepalive, yaml);
impl_connection_getter!(get_network_connection_persistent_keepalive_enabled, persistent_keepalive.enabled);
impl_connection_getter!(get_network_connection_persistent_keepalive_period, persistent_keepalive.period);
impl_connection_getter!(get_network_connection_allowed_ips_a_to_b, allowed_ips_a_to_b, yaml);
impl_connection_getter!(get_network_connection_allowed_ips_b_to_a, allowed_ips_b_to_a, yaml);

// Network indexed getters for reservations (using macros)
impl_reservation_getter!(get_network_reservation);
impl_reservation_getter!(get_network_reservation_peer_id, peer_id);
impl_reservation_getter!(get_network_reservation_valid_until, valid_until);

// ============================================================================
// Configuration Getter Functions
// ============================================================================


// Macro to generate get functions for config fields
macro_rules! impl_config_getter {
    // For simple types that implement Display
    ($fn_name:ident, $($field:ident).+) => {
        pub fn $fn_name() -> Result<(), ConfigCommandError> {
            let config = conf::util::get_config()?;
            println!("{}", config.$($field).+);
            Ok(())
        }
    };
    // For PathBuf types that need .display()
    ($fn_name:ident, $($field:ident).+, display) => {
        pub fn $fn_name() -> Result<(), ConfigCommandError> {
            let config = conf::util::get_config()?;
            println!("{}", config.$($field).+.display());
            Ok(())
        }
    };
    // For structs that should be serialized as YAML
    ($fn_name:ident, $($field:ident).+, yaml) => {
        pub fn $fn_name() -> Result<(), ConfigCommandError> {
            let config = conf::util::get_config()?;
            print_as_yaml(&config.$($field).+)
        }
    };
}

// Get functions - generated using macro
// Agent struct getters
impl_config_getter!(get_agent, agent, yaml);
impl_config_getter!(get_agent_web, agent.web, yaml);
impl_config_getter!(get_agent_web_http, agent.web.http, yaml);
impl_config_getter!(get_agent_web_https, agent.web.https, yaml);
impl_config_getter!(get_agent_web_password, agent.web.password, yaml);
impl_config_getter!(get_agent_vpn, agent.vpn, yaml);

// Agent individual field getters
impl_config_getter!(get_agent_web_address, agent.web.address);
impl_config_getter!(get_agent_web_http_enabled, agent.web.http.enabled);
impl_config_getter!(get_agent_web_http_port, agent.web.http.port);
impl_config_getter!(get_agent_web_https_enabled, agent.web.https.enabled);
impl_config_getter!(get_agent_web_https_port, agent.web.https.port);
impl_config_getter!(get_agent_web_https_tls_cert, agent.web.https.tls_cert, display);
impl_config_getter!(get_agent_web_https_tls_key, agent.web.https.tls_key, display);
impl_config_getter!(get_agent_web_password_enabled, agent.web.password.enabled);
impl_config_getter!(get_agent_web_password_hash, agent.web.password.hash);
impl_config_getter!(get_agent_vpn_enabled, agent.vpn.enabled);
impl_config_getter!(get_agent_vpn_port, agent.vpn.port);
impl_config_getter!(get_agent_vpn_wg, agent.vpn.wg, display);
impl_config_getter!(get_agent_vpn_wg_userspace, agent.vpn.wg_userspace, yaml);
impl_config_getter!(get_agent_vpn_wg_userspace_enabled, agent.vpn.wg_userspace.enabled);
impl_config_getter!(get_agent_vpn_wg_userspace_binary, agent.vpn.wg_userspace.binary, display);

// Network struct getter
impl_config_getter!(get_network, network, yaml);

// Network individual field getters
impl_config_getter!(get_network_name, network.name);
impl_config_getter!(get_network_subnet, network.subnet);
impl_config_getter!(get_network_this_peer, network.this_peer);
impl_config_getter!(get_network_peers, network.peers, yaml);
impl_config_getter!(get_network_connections, network.connections, yaml);
impl_config_getter!(get_network_defaults, network.defaults, yaml);
impl_config_getter!(get_network_reservations, network.reservations, yaml);
impl_config_getter!(get_network_amnezia_parameters, network.amnezia_parameters, yaml);
impl_config_getter!(get_network_amnezia_parameters_enabled, network.amnezia_parameters.enabled);
impl_config_getter!(get_network_amnezia_parameters_s1, network.amnezia_parameters.s1);
impl_config_getter!(get_network_amnezia_parameters_s2, network.amnezia_parameters.s2);
impl_config_getter!(get_network_amnezia_parameters_h1, network.amnezia_parameters.h1);
impl_config_getter!(get_network_amnezia_parameters_h2, network.amnezia_parameters.h2);
impl_config_getter!(get_network_amnezia_parameters_h3, network.amnezia_parameters.h3);
impl_config_getter!(get_network_amnezia_parameters_h4, network.amnezia_parameters.h4);
impl_config_getter!(get_network_updated_at, network.updated_at);

// Network defaults field getters
impl_config_getter!(get_network_defaults_peer, network.defaults.peer, yaml);
impl_config_getter!(get_network_defaults_peer_amnezia_parameters, network.defaults.peer.amnezia_parameters, yaml);
impl_config_getter!(get_network_defaults_peer_amnezia_parameters_jc, network.defaults.peer.amnezia_parameters.jc);
impl_config_getter!(get_network_defaults_peer_amnezia_parameters_jmin, network.defaults.peer.amnezia_parameters.jmin);
impl_config_getter!(get_network_defaults_peer_amnezia_parameters_jmax, network.defaults.peer.amnezia_parameters.jmax);
impl_config_getter!(get_network_defaults_connection, network.defaults.connection, yaml);
