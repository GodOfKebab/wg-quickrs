use ipnet::Ipv4Net;
use std::net::Ipv4Addr;
use chrono::Utc;
use serde::Deserialize;
use serde::de::IntoDeserializer;
use uuid::Uuid;
use crate::types::network::*;
use crate::validation::error::{ValidationError, ValidationResult};


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
    let address_ipv4 = address.parse().map_err(|_| ValidationError::NotIPv4Address())?;
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

