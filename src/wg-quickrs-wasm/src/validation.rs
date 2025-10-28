use ipnet::Ipv4Net;
use serde::{Deserialize};
use std::net::{Ipv4Addr};
use crate::types::conf::{AllowedIPs, Dns, Endpoint, EndpointAddress, Icon, Mtu, Network, PersistentKeepalive, Script, WireGuardKey};
use chrono::Utc;
use serde::de::IntoDeserializer;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, PartialEq, Debug)]
pub enum ValidationError {
    #[error("address is not IPv4")]
    NotIPv4Address(),
    #[error("subnet is not in CIDR format")]
    NotCIDR(),
    #[error("name cannot be empty")]
    EmptyName(),
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
    #[error("MTU is invalid (1-9999)")]
    InvalidMtu(),
    #[error("icon cannot be empty when enabled")]
    EmptyIcon(),
    #[error("key is not a valid WireGuard key (32 bytes, base64 encoded)")]
    NotWireGuardKey(),
    #[error("script needs to end with a semicolon")]
    ScriptMissingSemicolon(),
    #[error("PersistentKeepalive is invalid")]
    InvalidPersistentKeepalive(),
    #[error("AllowedIPs is not in CIDR format")]
    InvalidAllowedIPs(),

}
pub type ValidationResult<T> = Result<T, ValidationError>;

pub fn validate_address(address: &str) -> ValidationResult<Ipv4Addr> {
    address.parse().map_err(|_| ValidationError::NotIPv4Address())
}

pub fn validate_subnet(subnet: &str) -> ValidationResult<Ipv4Net> {
    subnet.parse().map_err(|_| ValidationError::NotCIDR())
}

pub fn validate_name(name: &str) -> ValidationResult<&str> {
    if name.is_empty() {
        return Err(ValidationError::EmptyName());
    }
    Ok(name)
}

pub fn validate_internal_address(address: &str, network: &Network) -> ValidationResult<Ipv4Addr> {
    let address_ipv4 = validate_address(address)?;
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

pub fn validate_endpoint(enabled: bool, endpoint: &str) -> ValidationResult<Endpoint> {
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

pub fn validate_dns(enabled: bool, dns: &str) -> ValidationResult<Dns> {
    if !enabled {
        return Ok(Dns { enabled, addresses: Vec::new() });
    }
    let addresses = dns.split(',')
        .map(|address| address.trim().parse().map_err(|_| ValidationError::NotIPv4Address()))
        .collect::<ValidationResult<Vec<_>>>()?;

    Ok(Dns { enabled, addresses })
}

pub fn validate_mtu(enabled: bool, mtu: &str) -> ValidationResult<Mtu> {
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

pub fn validate_kind(kind: &str) -> ValidationResult<&str> {
    Ok(kind)
}

pub fn validate_icon(enabled: bool, icon: &str) -> ValidationResult<Icon> {
    if enabled && icon.is_empty() {
        return Err(ValidationError::EmptyIcon());
    }
    Ok(Icon{ enabled, src: icon.to_string() })
}

pub fn validate_wg_key(key: &str) -> ValidationResult<WireGuardKey> {
    WireGuardKey::deserialize(key.into_deserializer())
        .map_err(|_: serde::de::value::Error| ValidationError::NotWireGuardKey())
}

pub fn validate_script(enabled: bool, script: &str) -> ValidationResult<Script> {
    if enabled && !script.trim().ends_with(';') {
        return Err(ValidationError::ScriptMissingSemicolon());
    }
    Ok(Script { enabled, script: script.to_string() })
}

pub fn validate_persistent_keepalive(enabled: bool, persistent_keepalive: &str) -> ValidationResult<PersistentKeepalive> {
    let period = if enabled {
        persistent_keepalive.parse::<u16>()
            .map_err(|_| ValidationError::InvalidPersistentKeepalive())?
    } else {
        0
    };

    Ok(PersistentKeepalive { enabled, period })
}

pub fn validate_allowed_ips(allowed_ips: &str) -> ValidationResult<AllowedIPs> {
    let ips = allowed_ips.split(',')
        .map(|cidr| cidr.trim().parse::<Ipv4Net>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ValidationError::InvalidAllowedIPs())?;

    Ok(ips)
}

