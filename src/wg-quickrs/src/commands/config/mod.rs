mod toggle;
mod get;
mod set;
mod list;
mod remove;
mod reset;
mod add;

use std::io;
use std::net::Ipv4Addr;
use std::str::FromStr;
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;
use wg_quickrs_cli::config::*;
use wg_quickrs_cli::config::enable::*;
use wg_quickrs_cli::config::disable::*;
use wg_quickrs_cli::config::set::*;
use wg_quickrs_cli::config::reset::*;
use wg_quickrs_cli::config::get::*;
use wg_quickrs_cli::config::list::*;
use wg_quickrs_cli::config::remove::*;
use wg_quickrs_cli::config::add::*;
use wg_quickrs_lib::types::network::ConnectionId;
use wg_quickrs_lib::validation::error::ValidationError;
use crate::commands::config::toggle::*;
use crate::commands::config::get::*;
use crate::commands::config::set::*;
use crate::commands::config::list::*;
use crate::commands::config::remove::*;
use crate::commands::config::reset::*;
use crate::commands::config::add::*;
use crate::conf;
use crate::conf::util::ConfUtilError;

#[derive(Error, Debug)]
pub enum ConfigCommandError {
    #[error("{0}")]
    PasswordHash(String),
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error(transparent)]
    ConfUtilError(#[from] ConfUtilError),
    #[error("cannot enable firewall gateway: gateway is not set")]
    GatewayNotSet(),
    #[error("failed to read input: {0}")]
    ReadFailed(#[from] io::Error),
    #[error("failed to serialize to YAML: {0}")]
    YamlSerialization(#[from] serde_yml::Error),
    #[error("peer not found: {0}")]
    PeerNotFound(Uuid),
    #[error("connection not found: {0}")]
    ConnectionNotFound(String),
    #[error("reservation not found: {0}")]
    ReservationNotFound(Ipv4Addr),
    #[error("invalid connection id format: {0}")]
    InvalidConnectionId(String),
    #[error("invalid uuid format: {0}")]
    InvalidUuid(#[from] uuid::Error),
    #[error("missing required argument: {0}")]
    MissingArgument(String),
    #[error("cannot remove this_peer: {0}")]
    CannotRemoveThisPeer(Uuid),
}

impl From<argon2::password_hash::Error> for ConfigCommandError {
    fn from(err: argon2::password_hash::Error) -> Self {
        ConfigCommandError::PasswordHash(err.to_string())
    }
}

// A helper function to print any serializable struct as YAML
pub fn print_as_yaml<T: Serialize>(value: &T) -> Result<(), ConfigCommandError> {
    let yaml = serde_yml::to_string(value)?;
    print!("{}", yaml);
    Ok(())
}

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

// Command handler - dispatches config commands to appropriate functions
pub fn handle_config_command(target: &ConfigCommands) -> Result<(), ConfigCommandError> {
    match target {
        ConfigCommands::Enable { target } => match target {
            EnableCommands::Agent { target } => match target {
                EnableAgentCommands::Web { target } => match target {
                    EnableAgentWebCommands::Http => toggle_agent_web_http(true),
                    EnableAgentWebCommands::Https => toggle_agent_web_https(true),
                    EnableAgentWebCommands::Password => toggle_agent_web_password(true),
                },
                EnableAgentCommands::Vpn => toggle_agent_vpn(true),
            },
            EnableCommands::Network { target } => match target {
                EnableNetworkCommands::Peer { id, target } => match target {
                    EnablePeerCommands::Endpoint => enable_peer_endpoint(id),
                    EnablePeerCommands::Icon => enable_peer_icon(id),
                    EnablePeerCommands::Dns => enable_peer_dns(id),
                    EnablePeerCommands::Mtu => enable_peer_mtu(id),
                },
                EnableNetworkCommands::Connection { id } => enable_connection(id),
                EnableNetworkCommands::Defaults { target } => match target {
                    EnableDefaultsCommands::Peer { target } => match target {
                        EnableDefaultsPeerCommands::Icon => enable_defaults_peer_icon(),
                        EnableDefaultsPeerCommands::Dns => enable_defaults_peer_dns(),
                        EnableDefaultsPeerCommands::Mtu => enable_defaults_peer_mtu(),
                    },
                    EnableDefaultsCommands::Connection { target } => match target {
                        EnableDefaultsConnectionCommands::PersistentKeepalive => enable_defaults_connection_persistent_keepalive(),
                    },
                },
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
            },
            DisableCommands::Network { target } => match target {
                DisableNetworkCommands::Peer { id, target } => match target {
                    DisablePeerCommands::Endpoint => disable_peer_endpoint(id),
                    DisablePeerCommands::Icon => disable_peer_icon(id),
                    DisablePeerCommands::Dns => disable_peer_dns(id),
                    DisablePeerCommands::Mtu => disable_peer_mtu(id),
                },
                DisableNetworkCommands::Connection { id } => disable_connection(id),
                DisableNetworkCommands::Defaults { target } => match target {
                    DisableDefaultsCommands::Peer { target } => match target {
                        DisableDefaultsPeerCommands::Icon => disable_defaults_peer_icon(),
                        DisableDefaultsPeerCommands::Dns => disable_defaults_peer_dns(),
                        DisableDefaultsPeerCommands::Mtu => disable_defaults_peer_mtu(),
                    },
                    DisableDefaultsCommands::Connection { target } => match target {
                        DisableDefaultsConnectionCommands::PersistentKeepalive => disable_defaults_connection_persistent_keepalive(),
                    },
                },
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
            },
            SetCommands::Network { target } => match target {
                SetNetworkCommands::Name { name } => set_network_name(name.clone()),
                SetNetworkCommands::Subnet { subnet } => set_network_subnet(subnet),
                SetNetworkCommands::Peer { id, target } => match target {
                    SetPeerCommands::Name { name } => set_peer_name(id, name.clone()),
                    SetPeerCommands::Address { address } => set_peer_address(id, *address),
                    SetPeerCommands::Endpoint { endpoint } => set_peer_endpoint(id, endpoint),
                    SetPeerCommands::Kind { kind } => set_peer_kind(id, kind),
                    SetPeerCommands::Icon { src } => set_peer_icon(id, src),
                    SetPeerCommands::Dns { addresses } => set_peer_dns(id, addresses),
                    SetPeerCommands::Mtu { value } => set_peer_mtu(id, *value),
                },
                SetNetworkCommands::Connection { id, target } => match target {
                    SetConnectionCommands::AllowedIpsAToB { ips } => set_connection_allowed_ips_a_to_b(id, ips),
                    SetConnectionCommands::AllowedIpsBToA { ips } => set_connection_allowed_ips_b_to_a(id, ips),
                    SetConnectionCommands::PersistentKeepalive { period } => set_connection_persistent_keepalive(id, *period),
                },
                SetNetworkCommands::Defaults { target } => match target {
                    SetDefaultsCommands::Peer { target } => match target {
                        SetDefaultsPeerCommands::Kind { kind } => set_defaults_peer_kind(kind),
                        SetDefaultsPeerCommands::Icon { src } => set_defaults_peer_icon(src),
                        SetDefaultsPeerCommands::Dns { addresses } => set_defaults_peer_dns(addresses),
                        SetDefaultsPeerCommands::Mtu { value } => set_defaults_peer_mtu(*value),
                    },
                    SetDefaultsCommands::Connection { target } => match target {
                        SetDefaultsConnectionCommands::PersistentKeepalive { period } => set_defaults_connection_persistent_keepalive(*period),
                    },
                },
            },
        },
        ConfigCommands::Reset { target } => match target {
            ResetCommands::Agent { target } => match target {
                ResetAgentCommands::Web { target } => match target {
                    ResetAgentWebCommands::Password { password } => {
                        reset_web_password(password)
                    },
                },
            },
            ResetCommands::Network { target } => match target {
                ResetNetworkCommands::Peer { id, target } => match target {
                    ResetPeerCommands::PrivateKey => reset_peer_private_key(id),
                },
                ResetNetworkCommands::Connection { id, target } => match target {
                    ResetConnectionCommands::PreSharedKey => reset_connection_pre_shared_key(id),
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
        ConfigCommands::List { target } => match target {
            ListCommands::Peers => list_network_peers(),
            ListCommands::Connections => list_network_connections(),
            ListCommands::Reservations => list_network_reservations(),
        },
        ConfigCommands::Remove { target } => match target {
            RemoveCommands::Peer { id } => remove_network_peer(id),
            RemoveCommands::Connection { id } => remove_network_connection(id),
            RemoveCommands::Reservation { address } => remove_network_reservation(address),
        },
        ConfigCommands::Add { target } => match target {
            AddCommands::Peer { options } => add_peer(options),
            AddCommands::Connection { options } => add_connection(options),
        },
    }
}

