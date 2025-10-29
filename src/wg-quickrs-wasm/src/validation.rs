use ipnet::Ipv4Net;
use serde::Deserialize;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use crate::types::conf::{AllowedIPs, Dns, Endpoint, EndpointAddress, Icon, Mtu, Network, PersistentKeepalive, Script, WireGuardKey};
use chrono::Utc;
use serde::de::IntoDeserializer;
use thiserror::Error;
use uuid::Uuid;
use crate::validation_helpers;
#[cfg(not(target_arch = "wasm32"))]
use get_if_addrs::Interface;

#[derive(Error, PartialEq, Debug)]
pub enum ValidationError {
    #[error("address is not IPv4")]
    NotIPv4Address(),
    #[error("port is not a valid number (1-65535)")]
    NotPortNumber(),
    #[error("TLS file is not found")]
    TlsFileNotFound(),
    #[error("TLS path is not a file (it is a directory or a symlink)")]
    TlsFileNotAFile(),
    #[error("gateway {0} is not found (possible options: {1})")]
    InterfaceNotFound(String, String),
    #[error("firewall utility {0} is not found (possible options: {1})")]
    FirewallUtilityNotFound(String, String),
    #[error("subnet is not in CIDR format")]
    NotCIDR(),
    #[error("Uuid is invalid (not in v4 format)")]
    InvalidUuid(),
    #[error("network name cannot be empty")]
    EmptyNetworkName(),
    #[error("peer name cannot be empty")]
    EmptyPeerName(),
    #[error("address is not in the network subnet")]
    AddressNotInSubnet(),
    #[error("address is the subnet's network address and cannot be assigned")]
    AddressIsSubnetNetwork(),
    #[error("address is the subnet's broadcast address and cannot be assigned")]
    AddressIsSubnetBroadcast(),
    #[error("address is already taken by {0} ({1})")]
    AddressIsTaken(Uuid, String),
    #[error("address is already reserved for another peer")]
    AddressIsReserved(),
    #[error("Endpoint is invalid")]
    InvalidEndpoint(),
    #[error("Endpoint port is invalid")]
    InvalidEndpointPort(),
    #[error("icon cannot be empty when enabled")]
    EmptyIcon(),
    #[error("MTU is invalid (1-9999)")]
    InvalidMtu(),
    #[error("script needs to end with a semicolon")]
    ScriptMissingSemicolon(),
    #[error("key is not a valid WireGuard key (32 bytes, base64 encoded)")]
    NotWireGuardKey(),
    #[error("PersistentKeepalive is invalid")]
    InvalidPersistentKeepalive(),
    #[error("AllowedIPs is not in CIDR format")]
    InvalidAllowedIPs(),
}
pub type ValidationResult<T> = Result<T, ValidationError>;

// Agent Fields

pub fn validate_ipv4_address(address: &str) -> ValidationResult<Ipv4Addr> {
    address.parse().map_err(|_| ValidationError::NotIPv4Address())
}

pub fn validate_port(port: &str) -> ValidationResult<u16> {
    port.parse().map_err(|_| ValidationError::NotPortNumber())
}

pub fn validate_tls_file(config_folder: &Path, tls_file: &str) -> ValidationResult<PathBuf> {
    let tls_file_path = config_folder.join(tls_file);

    if !tls_file_path.exists() {
        return Err(ValidationError::TlsFileNotFound());
    }

    if !tls_file_path.is_file() {
        return Err(ValidationError::TlsFileNotAFile());
    }

    Ok(tls_file_path)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn validate_fw_gateway(fw_gateway: &str) -> ValidationResult<Interface> {
    let interfaces = validation_helpers::get_interfaces();

    // Try to find interface by name
    if let Some(iface) = interfaces.iter().find(|iface| iface.name == fw_gateway) {
        return Ok(iface.clone());
    }

    // If not found, prepare an error with available options
    let available: Vec<String> = interfaces
        .iter()
        .map(|iface| format!("{} ({})", iface.name, iface.ip()))
        .collect();

    Err(ValidationError::InterfaceNotFound(fw_gateway.to_string(), format!("[{}]", available.join(", "))))
}

pub fn validate_fw_utility(fw_utility: &str) -> ValidationResult<PathBuf> {
    let bin_path = Path::new(fw_utility);

    // Check if a path exists and is a file
    if bin_path.exists() && bin_path.is_file() {
        return Ok(bin_path.to_path_buf());
    }

    // Not found - create helpful error message
    let options = validation_helpers::firewall_utility_options();
    Err(ValidationError::FirewallUtilityNotFound(fw_utility.to_string(), format!("[{}]", options.join(", "))))
}

// Network Fields

pub fn validate_network_name(network_name: &str) -> ValidationResult<&str> {
    if network_name.is_empty() {
        return Err(ValidationError::EmptyNetworkName());
    }
    Ok(network_name)
}

pub fn validate_ipv4_subnet(subnet: &str) -> ValidationResult<Ipv4Net> {
    subnet.parse().map_err(|_| ValidationError::NotCIDR())
}

pub fn validate_peer_id(peer_id: &str) -> ValidationResult<Uuid> {
    peer_id.parse().map_err(|_| ValidationError::InvalidUuid())
}

// Network.Peer Fields

pub fn validate_peer_name(name: &str) -> ValidationResult<&str> {
    if name.is_empty() {
        return Err(ValidationError::EmptyPeerName());
    }
    Ok(name)
}

pub fn validate_peer_address(address: &str, network: &Network) -> ValidationResult<Ipv4Addr> {
    let address_ipv4 = validate_ipv4_address(address)?;
    if !network.subnet.contains(&address_ipv4) {
        return Err(ValidationError::AddressNotInSubnet());
    }
    if address_ipv4 == network.subnet.network() {
        return Err(ValidationError::AddressIsSubnetNetwork());
    }
    if address_ipv4 == network.subnet.broadcast() {
        return Err(ValidationError::AddressIsSubnetBroadcast());
    }
    if let Some((peer_id, peer)) = network.peers.iter().find(|(_, p)| p.address == address_ipv4) {
        return Err(ValidationError::AddressIsTaken(*peer_id, peer.name.clone()));
    }
    if network.reservations.get(&address_ipv4).is_some_and(|res| Utc::now() < res.valid_until) {
        return Err(ValidationError::AddressIsReserved());
    }
    Ok(address_ipv4)
}

pub fn validate_peer_endpoint(enabled: bool, endpoint: &str) -> ValidationResult<Endpoint> {
    if !enabled {
        return Ok(Endpoint {
            enabled,
            address: EndpointAddress::None,
            port: 0,
        });
    }

    // Split by last ':' to separate address and port
    let (address_str, port_str) = endpoint.rsplit_once(':')
        .ok_or(ValidationError::InvalidEndpoint())?;

    let port = port_str.parse::<u16>()
        .map_err(|_| ValidationError::InvalidEndpointPort())?;

    // Try parsing as IPv4 first, then fall back to the hostname
    let address = if let Ok(ipv4) = address_str.parse::<Ipv4Addr>() {
        EndpointAddress::Ipv4(ipv4)
    } else if hostname_validator::is_valid(address_str) {
        EndpointAddress::Hostname(address_str.to_string())
    } else {
        return Err(ValidationError::InvalidEndpoint());
    };

    Ok(Endpoint {
        enabled,
        address,
        port,
    })
}

pub fn validate_peer_kind(kind: &str) -> ValidationResult<&str> {
    // no validation
    Ok(kind)
}

pub fn validate_peer_icon(enabled: bool, src: &str) -> ValidationResult<Icon> {
    if enabled && src.is_empty() {
        return Err(ValidationError::EmptyIcon());
    }
    Ok(Icon{ enabled, src: src.to_string() })
}


pub fn validate_peer_dns(enabled: bool, dns: &str) -> ValidationResult<Dns> {
    if !enabled {
        return Ok(Dns { enabled, addresses: Vec::new() });
    }
    let addresses = dns.split(',')
        .map(|address| address.trim().parse().map_err(|_| ValidationError::NotIPv4Address()))
        .collect::<ValidationResult<Vec<_>>>()?;

    Ok(Dns { enabled, addresses })
}

pub fn validate_peer_mtu(enabled: bool, mtu: &str) -> ValidationResult<Mtu> {
    if !enabled {
        return Ok(Mtu { enabled, value: 0 });
    }

    let value = mtu.parse::<u16>()
        .map_err(|_| ValidationError::InvalidMtu())?;

    if value > 10000 {
        return Err(ValidationError::InvalidMtu());
    }

    Ok(Mtu { enabled, value })
}

pub fn validate_peer_script(enabled: bool, script: &str) -> ValidationResult<Script> {
    if enabled && !script.trim().ends_with(';') {
        return Err(ValidationError::ScriptMissingSemicolon());
    }
    Ok(Script { enabled, script: script.to_string() })
}

pub fn validate_wg_key(key: &str) -> ValidationResult<WireGuardKey> {
    WireGuardKey::deserialize(key.into_deserializer())
        .map_err(|_: serde::de::value::Error| ValidationError::NotWireGuardKey())
}

// Network.Connection Fields

pub fn validate_conn_persistent_keepalive(enabled: bool, persistent_keepalive: &str) -> ValidationResult<PersistentKeepalive> {
    let period = if enabled {
        persistent_keepalive.parse::<u16>()
            .map_err(|_| ValidationError::InvalidPersistentKeepalive())?
    } else {
        0
    };

    Ok(PersistentKeepalive { enabled, period })
}

pub fn validate_conn_allowed_ips(allowed_ips: &str) -> ValidationResult<AllowedIPs> {
    let ips = allowed_ips.split(',')
        .map(|cidr| cidr.trim().parse::<Ipv4Net>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ValidationError::InvalidAllowedIPs())?;

    Ok(ips)
}

