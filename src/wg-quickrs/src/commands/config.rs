use crate::commands::helpers;
use crate::conf;
use argon2::PasswordHash;
use wg_quickrs_cli::ResetWebPasswordOptions;
use wg_quickrs_lib::validation::agent::*;
use wg_quickrs_lib::validation::error::*;
use wg_quickrs_lib::types::network::ConnectionId;
use std::io;
use std::io::Write;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;
use crate::conf::util::ConfUtilError;
use crate::WG_QUICKRS_CONFIG_FOLDER;
use serde::Serialize;

#[derive(Error, Debug)]
pub enum ConfigCommandError {
    #[error("{0}")]
    PasswordHash(String),
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error(transparent)]
    ConfUtilError(#[from] ConfUtilError),
    #[error("Can't enable firewall gateway: Gateway is not set")]
    GatewayNotSet(),
    #[error("Failed to read input: {0}")]
    ReadFailed(#[from] io::Error),
    #[error("Failed to serialize to YAML: {0}")]
    YamlSerialization(#[from] serde_yml::Error),
    #[error("Peer not found: {0}")]
    PeerNotFound(Uuid),
    #[error("Connection not found: {0}")]
    ConnectionNotFound(String),
    #[error("Reservation not found: {0}")]
    ReservationNotFound(Ipv4Addr),
    #[error("Invalid connection ID format: {0}")]
    InvalidConnectionId(String),
    #[error("Invalid UUID format: {0}")]
    InvalidUuid(#[from] uuid::Error),
    #[error("Missing required argument: {0}")]
    MissingArgument(String),
}

impl From<argon2::password_hash::Error> for ConfigCommandError {
    fn from(err: argon2::password_hash::Error) -> Self {
        ConfigCommandError::PasswordHash(err.to_string())
    }
}

// ============================================================================
// Macros for reducing boilerplate
// ============================================================================

/// Macro for implementing toggle (enable/disable) functions
macro_rules! impl_toggle {
    // Without validation
    ($fn_name:ident, $($field:ident).+ => $log_format:expr) => {
        pub fn $fn_name(status: bool) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            log::info!(
                "{} {}",
                if status { "Enabling" } else { "Disabling" },
                $log_format(&config)
            );
            config.$($field).+.enabled = status;
            conf::util::set_config(&mut config)?;
            Ok(())
        }
    };
    // With validation
    ($fn_name:ident, $($field:ident).+ => $log_format:expr, validate: $validator:expr) => {
        pub fn $fn_name(status: bool) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            log::info!(
                "{} {}",
                if status { "Enabling" } else { "Disabling" },
                $log_format(&config)
            );
            if status {
                $validator(&config)?;
            }
            config.$($field).+.enabled = status;
            conf::util::set_config(&mut config)?;
            Ok(())
        }
    };
}

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

// ============================================================================
// Agent Web Configuration Functions
// ============================================================================

impl_setter!(set_agent_web_address, Ipv4Addr, agent.web.address, "agent address");

impl_toggle!(
    toggle_agent_web_http,
    agent.web.http =>
    |c: &wg_quickrs_lib::types::config::Config| format!("HTTP web server (port={})...", c.agent.web.http.port)
);

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

impl_toggle!(
    toggle_agent_web_https,
    agent.web.https =>
    |c: &wg_quickrs_lib::types::config::Config| format!(
        "HTTPS web server (port={}, tls_cert={}, tls_key={})...",
        c.agent.web.https.port,
        c.agent.web.https.tls_cert.display(),
        c.agent.web.https.tls_key.display()
    ),
    validate: |c: &wg_quickrs_lib::types::config::Config| -> Result<(), ConfigCommandError> {
        let wg_quickrs_conf_folder = WG_QUICKRS_CONFIG_FOLDER.get().unwrap();
        validate_tls_file(wg_quickrs_conf_folder, &c.agent.web.https.tls_cert)?;
        validate_tls_file(wg_quickrs_conf_folder, &c.agent.web.https.tls_key)?;
        Ok(())
    }
);

impl_port_setter!(set_agent_web_https_port, agent.web.https, "HTTPS");

impl_toggle!(
    toggle_agent_web_password,
    agent.web.password =>
    |_: &wg_quickrs_lib::types::config::Config| "password for the web server...".to_string(),
    validate: |c: &wg_quickrs_lib::types::config::Config| -> Result<(), ConfigCommandError> {
        PasswordHash::new(&c.agent.web.password.hash)?;
        Ok(())
    }
);

// ============================================================================
// Special Configuration Functions (not macro-generated)
// ============================================================================

pub fn reset_web_password(reset_web_password_opts: &ResetWebPasswordOptions) -> Result<(), ConfigCommandError> {
    // get the wireguard config a file path
    let mut config = conf::util::get_config()?;

    log::info!("Resetting the web password...");
    let password = match reset_web_password_opts.password.clone() {
        Some(pwd) => {
            log::warn!(
                "THIS IS HIGHLY INSECURE! Please set the password without the --password flag. The plaintext password could be visible in your shell history."
            );
            pwd
        }
        None => {
            print!("Enter your new password: ");
            io::stdout().flush().unwrap(); // Ensure the prompt is shown before waiting for input

            let mut pwd = String::new();
            io::stdin().read_line(&mut pwd).map_err(|e| ConfigCommandError::ReadFailed(e))?;
            pwd.trim().to_string()
        }
    };
    let password_hash = helpers::calculate_password_hash(password.trim())?;

    config.agent.web.password.hash = password_hash;
    conf::util::set_config(&mut config)?;
    Ok(())
}

// ============================================================================
// Agent VPN Configuration Functions
// ============================================================================

impl_toggle!(
    toggle_agent_vpn,
    agent.vpn =>
    |c: &wg_quickrs_lib::types::config::Config| format!("VPN server (port={})...", c.agent.vpn.port)
);

impl_port_setter!(set_agent_vpn_port, agent.vpn, "VPN");

// ============================================================================
// Agent Firewall Configuration Functions
// ============================================================================

impl_toggle!(
    toggle_agent_firewall,
    agent.firewall =>
    |c: &wg_quickrs_lib::types::config::Config| format!(
        "firewall setting up NAT and input rules (utility={})...",
        c.agent.firewall.utility.display()
    ),
    validate: |c: &wg_quickrs_lib::types::config::Config| -> Result<(), ConfigCommandError> {
        if c.agent.firewall.gateway.is_empty() {
            return Err(ConfigCommandError::GatewayNotSet());
        }
        validate_fw_utility(&c.agent.firewall.utility)?;
        Ok(())
    }
);

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

// ============================================================================
// Configuration Getter Functions
// ============================================================================

// Helper function to print any serializable struct as YAML
fn print_as_yaml<T: Serialize>(value: &T) -> Result<(), ConfigCommandError> {
    let yaml = serde_yml::to_string(value)?;
    print!("{}", yaml);
    Ok(())
}

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
impl_config_getter!(get_agent_firewall, agent.firewall, yaml);

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
impl_config_getter!(get_agent_firewall_enabled, agent.firewall.enabled);
impl_config_getter!(get_agent_firewall_utility, agent.firewall.utility, display);
impl_config_getter!(get_agent_firewall_gateway, agent.firewall.gateway);

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
impl_config_getter!(get_network_updated_at, network.updated_at);

// Network defaults field getters
impl_config_getter!(get_network_defaults_peer, network.defaults.peer, yaml);
impl_config_getter!(get_network_defaults_connection, network.defaults.connection, yaml);

// Helper function to parse ConnectionId from string
fn parse_connection_id(id_str: &str) -> Result<ConnectionId, ConfigCommandError> {
    let parts: Vec<&str> = id_str.split('*').collect();
    if parts.len() != 2 {
        return Err(ConfigCommandError::InvalidConnectionId(id_str.to_string()));
    }

    let a = Uuid::from_str(parts[0])?;
    let b = Uuid::from_str(parts[1])?;

    Ok(ConnectionId { a, b })
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
// List Functions - Human-readable output
// ============================================================================

/// Helper function to format EndpointAddress for display
fn format_endpoint_address(addr: &wg_quickrs_lib::types::network::EndpointAddress) -> String {
    use wg_quickrs_lib::types::network::EndpointAddress;
    match addr {
        EndpointAddress::None => "none".to_string(),
        EndpointAddress::Ipv4AndPort(ip_port) => format!("{}:{}", ip_port.ipv4, ip_port.port),
        EndpointAddress::HostnameAndPort(host_port) => format!("{}:{}", host_port.hostname, host_port.port),
    }
}

/// List all peers in human-readable format
/// Format: "name (peerid) @ address / {endpoint if enabled}"
pub fn list_network_peers() -> Result<(), ConfigCommandError> {
    let config = conf::util::get_config()?;

    if config.network.peers.is_empty() {
        println!("No peers found.");
        return Ok(());
    }

    for (peer_id, peer) in &config.network.peers {
        let endpoint_str = if peer.endpoint.enabled {
            format!(" / {}", format_endpoint_address(&peer.endpoint.address))
        } else {
            String::new()
        };

        println!("{} ({}) @ {}{}", peer.name, peer_id, peer.address, endpoint_str);
    }

    Ok(())
}

/// List all connections in human-readable format
/// Format: "name1<->name2 (connectionid)"
pub fn list_network_connections() -> Result<(), ConfigCommandError> {
    let config = conf::util::get_config()?;

    if config.network.connections.is_empty() {
        println!("No connections found.");
        return Ok(());
    }

    for (conn_id, _connection) in &config.network.connections {
        // Get peer names
        let peer_a_name = config.network.peers.get(&conn_id.a)
            .map(|p| p.name.as_str())
            .unwrap_or("unknown");
        let peer_b_name = config.network.peers.get(&conn_id.b)
            .map(|p| p.name.as_str())
            .unwrap_or("unknown");

        println!("{}<->{} ({}*{})", peer_a_name, peer_b_name, conn_id.a, conn_id.b);
    }

    Ok(())
}

// Command handler - dispatches config commands to appropriate functions
pub fn handle_config_command(target: &wg_quickrs_cli::ConfigCommands) -> Result<(), ConfigCommandError> {
    use wg_quickrs_cli::*;

    match target {
        ConfigCommands::Enable { target } => match target {
            EnableCommands::Agent { target } => match target {
                EnableAgentCommands::Web { target } => match target {
                    EnableAgentWebCommands::Http => toggle_agent_web_http(true),
                    EnableAgentWebCommands::Https => toggle_agent_web_https(true),
                    EnableAgentWebCommands::Password => toggle_agent_web_password(true),
                },
                EnableAgentCommands::Vpn => toggle_agent_vpn(true),
                EnableAgentCommands::Firewall => toggle_agent_firewall(true),
            },
        },
        ConfigCommands::Disable { target } => match target {
            DisableCommands::Agent { target } => match target {
                DisableAgentCommands::Web { target } => match target {
                    DisableAgentWebCommands::Http => toggle_agent_web_http(false),
                    DisableAgentWebCommands::Https => toggle_agent_web_https(false),
                    DisableAgentWebCommands::Password => toggle_agent_web_password(false),
                },
                DisableAgentCommands::Vpn => toggle_agent_vpn(false),
                DisableAgentCommands::Firewall => toggle_agent_firewall(false),
            },
        },
        ConfigCommands::Set { target } => match target {
            SetCommands::Agent { target } => match target {
                SetAgentCommands::Web { target } => match target {
                    SetAgentWebCommands::Address { value } => set_agent_web_address(value),
                    SetAgentWebCommands::Http { target } => match target {
                        SetAgentWebHttpCommands::Port { value } => set_agent_web_http_port(*value),
                    },
                    SetAgentWebCommands::Https { target } => match target {
                        SetAgentWebHttpsCommands::Port { value } => set_agent_web_https_port(*value),
                        SetAgentWebHttpsCommands::TlsCert { value } => set_agent_web_http_tls_cert(value),
                        SetAgentWebHttpsCommands::TlsKey { value } => set_agent_web_http_tls_key(value),
                    },
                },
                SetAgentCommands::Vpn { target } => match target {
                    SetAgentVpnCommands::Port { value } => set_agent_vpn_port(*value),
                },
                SetAgentCommands::Firewall { target } => match target {
                    SetAgentFirewallCommands::Utility { value } => set_agent_firewall_utility(value),
                    SetAgentFirewallCommands::Gateway { value } => set_agent_firewall_gateway(value),
                },
            },
        },
        ConfigCommands::Reset { target } => match target {
            ResetCommands::Agent { target } => match target {
                ResetAgentCommands::Web { target } => match target {
                    ResetAgentWebCommands::Password { password } => {
                        reset_web_password(&ResetWebPasswordOptions {
                            password: password.clone(),
                        })
                    },
                },
            },
        },
        ConfigCommands::Get { target } => match target {
            GetCommands::Agent { target } => match target {
                None => get_agent(),
                Some(agent_cmd) => match agent_cmd {
                    GetAgentCommands::Web { target } => match target {
                        None => get_agent_web(),
                        Some(web_cmd) => match web_cmd {
                            GetAgentWebCommands::Address => get_agent_web_address(),
                            GetAgentWebCommands::Http { target } => match target {
                                None => get_agent_web_http(),
                                Some(http_cmd) => match http_cmd {
                                    GetAgentWebHttpCommands::Enabled => get_agent_web_http_enabled(),
                                    GetAgentWebHttpCommands::Port => get_agent_web_http_port(),
                                },
                            },
                            GetAgentWebCommands::Https { target } => match target {
                                None => get_agent_web_https(),
                                Some(https_cmd) => match https_cmd {
                                    GetAgentWebHttpsCommands::Enabled => get_agent_web_https_enabled(),
                                    GetAgentWebHttpsCommands::Port => get_agent_web_https_port(),
                                    GetAgentWebHttpsCommands::TlsCert => get_agent_web_https_tls_cert(),
                                    GetAgentWebHttpsCommands::TlsKey => get_agent_web_https_tls_key(),
                                },
                            },
                            GetAgentWebCommands::Password { target } => match target {
                                None => get_agent_web_password(),
                                Some(pwd_cmd) => match pwd_cmd {
                                    GetAgentWebPasswordCommands::Enabled => get_agent_web_password_enabled(),
                                    GetAgentWebPasswordCommands::Hash => get_agent_web_password_hash(),
                                },
                            },
                        },
                    },
                    GetAgentCommands::Vpn { target } => match target {
                        None => get_agent_vpn(),
                        Some(vpn_cmd) => match vpn_cmd {
                            GetAgentVpnCommands::Enabled => get_agent_vpn_enabled(),
                            GetAgentVpnCommands::Port => get_agent_vpn_port(),
                        },
                    },
                    GetAgentCommands::Firewall { target } => match target {
                        None => get_agent_firewall(),
                        Some(fw_cmd) => match fw_cmd {
                            GetAgentFirewallCommands::Enabled => get_agent_firewall_enabled(),
                            GetAgentFirewallCommands::Utility => get_agent_firewall_utility(),
                            GetAgentFirewallCommands::Gateway => get_agent_firewall_gateway(),
                        },
                    },
                },
            },
            GetCommands::Network { target } => match target {
                None => get_network(),
                Some(network_cmd) => match network_cmd {
                    GetNetworkCommands::Name => get_network_name(),
                    GetNetworkCommands::Subnet => get_network_subnet(),
                    GetNetworkCommands::ThisPeer => get_network_this_peer(),
                    GetNetworkCommands::Peers { id, target } => match (id, target) {
                        (None, None) => get_network_peers(),
                        (Some(peer_id), None) => get_network_peer(peer_id),
                        (Some(peer_id), Some(peer_cmd)) => match peer_cmd {
                            GetNetworkPeersCommands::Name => get_network_peer_name(peer_id),
                            GetNetworkPeersCommands::Address => get_network_peer_address(peer_id),
                            GetNetworkPeersCommands::Endpoint { target } => match target {
                                None => get_network_peer_endpoint(peer_id),
                                Some(endpoint_cmd) => match endpoint_cmd {
                                    GetNetworkPeersEndpointCommands::Enabled => get_network_peer_endpoint_enabled(peer_id),
                                    GetNetworkPeersEndpointCommands::Address => get_network_peer_endpoint_address(peer_id),
                                },
                            },
                            GetNetworkPeersCommands::Kind => get_network_peer_kind(peer_id),
                            GetNetworkPeersCommands::Icon { target } => match target {
                                None => get_network_peer_icon(peer_id),
                                Some(icon_cmd) => match icon_cmd {
                                    GetNetworkPeersIconCommands::Enabled => get_network_peer_icon_enabled(peer_id),
                                    GetNetworkPeersIconCommands::Src => get_network_peer_icon_src(peer_id),
                                },
                            },
                            GetNetworkPeersCommands::Dns { target } => match target {
                                None => get_network_peer_dns(peer_id),
                                Some(dns_cmd) => match dns_cmd {
                                    GetNetworkPeersDnsCommands::Enabled => get_network_peer_dns_enabled(peer_id),
                                    GetNetworkPeersDnsCommands::Addresses => get_network_peer_dns_addresses(peer_id),
                                },
                            },
                            GetNetworkPeersCommands::Mtu { target } => match target {
                                None => get_network_peer_mtu(peer_id),
                                Some(mtu_cmd) => match mtu_cmd {
                                    GetNetworkPeersMtuCommands::Enabled => get_network_peer_mtu_enabled(peer_id),
                                    GetNetworkPeersMtuCommands::Value => get_network_peer_mtu_value(peer_id),
                                },
                            },
                            GetNetworkPeersCommands::Scripts => get_network_peer_scripts(peer_id),
                            GetNetworkPeersCommands::PrivateKey => get_network_peer_private_key(peer_id),
                            GetNetworkPeersCommands::CreatedAt => get_network_peer_created_at(peer_id),
                            GetNetworkPeersCommands::UpdatedAt => get_network_peer_updated_at(peer_id),
                        },
                        (None, Some(_)) => {
                            Err(ConfigCommandError::MissingArgument("Peer ID is required when accessing peer fields".to_string()))
                        }
                    },
                    GetNetworkCommands::Connections { id, target } => match (id, target) {
                        (None, None) => get_network_connections(),
                        (Some(conn_id), None) => get_network_connection(conn_id),
                        (Some(conn_id), Some(conn_cmd)) => match conn_cmd {
                            GetNetworkConnectionsCommands::Enabled => get_network_connection_enabled(conn_id),
                            GetNetworkConnectionsCommands::PreSharedKey => get_network_connection_pre_shared_key(conn_id),
                            GetNetworkConnectionsCommands::PersistentKeepalive { target } => match target {
                                None => get_network_connection_persistent_keepalive(conn_id),
                                Some(ka_cmd) => match ka_cmd {
                                    GetNetworkConnectionsPersistentKeepaliveCommands::Enabled => get_network_connection_persistent_keepalive_enabled(conn_id),
                                    GetNetworkConnectionsPersistentKeepaliveCommands::Period => get_network_connection_persistent_keepalive_period(conn_id),
                                },
                            },
                            GetNetworkConnectionsCommands::AllowedIpsAToB => get_network_connection_allowed_ips_a_to_b(conn_id),
                            GetNetworkConnectionsCommands::AllowedIpsBToA => get_network_connection_allowed_ips_b_to_a(conn_id),
                        },
                        (None, Some(_)) => {
                            Err(ConfigCommandError::MissingArgument("Connection ID is required when accessing connection fields".to_string()))
                        }
                    },
                    GetNetworkCommands::Defaults { target } => match target {
                        None => get_network_defaults(),
                        Some(defaults_cmd) => match defaults_cmd {
                            GetNetworkDefaultsCommands::Peer { target } => match target {
                                None => get_network_defaults_peer(),
                                Some(peer_cmd) => match peer_cmd {
                                    GetNetworkDefaultsPeerCommands::Endpoint { target } => {
                                        // For defaults, we don't use indexed access, just direct field access
                                        let config = conf::util::get_config()?;
                                        match target {
                                            None => print_as_yaml(&config.network.defaults.peer.endpoint),
                                            Some(endpoint_cmd) => match endpoint_cmd {
                                                GetNetworkPeersEndpointCommands::Enabled => {
                                                    println!("{}", config.network.defaults.peer.endpoint.enabled);
                                                    Ok(())
                                                },
                                                GetNetworkPeersEndpointCommands::Address => {
                                                    print_as_yaml(&config.network.defaults.peer.endpoint.address)
                                                },
                                            },
                                        }
                                    },
                                    GetNetworkDefaultsPeerCommands::Kind => {
                                        let config = conf::util::get_config()?;
                                        println!("{}", config.network.defaults.peer.kind);
                                        Ok(())
                                    },
                                    GetNetworkDefaultsPeerCommands::Icon { target } => {
                                        let config = conf::util::get_config()?;
                                        match target {
                                            None => print_as_yaml(&config.network.defaults.peer.icon),
                                            Some(icon_cmd) => match icon_cmd {
                                                GetNetworkPeersIconCommands::Enabled => {
                                                    println!("{}", config.network.defaults.peer.icon.enabled);
                                                    Ok(())
                                                },
                                                GetNetworkPeersIconCommands::Src => {
                                                    println!("{}", config.network.defaults.peer.icon.src);
                                                    Ok(())
                                                },
                                            },
                                        }
                                    },
                                    GetNetworkDefaultsPeerCommands::Dns { target } => {
                                        let config = conf::util::get_config()?;
                                        match target {
                                            None => print_as_yaml(&config.network.defaults.peer.dns),
                                            Some(dns_cmd) => match dns_cmd {
                                                GetNetworkPeersDnsCommands::Enabled => {
                                                    println!("{}", config.network.defaults.peer.dns.enabled);
                                                    Ok(())
                                                },
                                                GetNetworkPeersDnsCommands::Addresses => {
                                                    print_as_yaml(&config.network.defaults.peer.dns.addresses)
                                                },
                                            },
                                        }
                                    },
                                    GetNetworkDefaultsPeerCommands::Mtu { target } => {
                                        let config = conf::util::get_config()?;
                                        match target {
                                            None => print_as_yaml(&config.network.defaults.peer.mtu),
                                            Some(mtu_cmd) => match mtu_cmd {
                                                GetNetworkPeersMtuCommands::Enabled => {
                                                    println!("{}", config.network.defaults.peer.mtu.enabled);
                                                    Ok(())
                                                },
                                                GetNetworkPeersMtuCommands::Value => {
                                                    println!("{}", config.network.defaults.peer.mtu.value);
                                                    Ok(())
                                                },
                                            },
                                        }
                                    },
                                    GetNetworkDefaultsPeerCommands::Scripts => {
                                        let config = conf::util::get_config()?;
                                        print_as_yaml(&config.network.defaults.peer.scripts)
                                    },
                                },
                            },
                            GetNetworkDefaultsCommands::Connection { target } => match target {
                                None => get_network_defaults_connection(),
                                Some(conn_cmd) => match conn_cmd {
                                    GetNetworkDefaultsConnectionCommands::PersistentKeepalive { target } => {
                                        let config = conf::util::get_config()?;
                                        match target {
                                            None => print_as_yaml(&config.network.defaults.connection.persistent_keepalive),
                                            Some(ka_cmd) => match ka_cmd {
                                                GetNetworkConnectionsPersistentKeepaliveCommands::Enabled => {
                                                    println!("{}", config.network.defaults.connection.persistent_keepalive.enabled);
                                                    Ok(())
                                                },
                                                GetNetworkConnectionsPersistentKeepaliveCommands::Period => {
                                                    println!("{}", config.network.defaults.connection.persistent_keepalive.period);
                                                    Ok(())
                                                },
                                            },
                                        }
                                    },
                                },
                            },
                        },
                    },
                    GetNetworkCommands::Reservations { ip, target } => match (ip, target) {
                        (None, None) => get_network_reservations(),
                        (Some(res_ip), None) => get_network_reservation(res_ip),
                        (Some(res_ip), Some(res_cmd)) => match res_cmd {
                            GetNetworkReservationsCommands::PeerId => get_network_reservation_peer_id(res_ip),
                            GetNetworkReservationsCommands::ValidUntil => get_network_reservation_valid_until(res_ip),
                        },
                        (None, Some(_)) => {
                            Err(ConfigCommandError::MissingArgument("IP address is required when accessing reservation fields".to_string()))
                        }
                    },
                    GetNetworkCommands::UpdatedAt => get_network_updated_at(),
                },
            },
        },
        ConfigCommands::Ls { target } => match target {
            LsCommands::Peers => list_network_peers(),
            LsCommands::Connections => list_network_connections(),
        },
    }
}

