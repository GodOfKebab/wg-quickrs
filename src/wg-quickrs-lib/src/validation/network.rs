use ipnet::Ipv4Net;
use std::net::Ipv4Addr;
use chrono::Utc;
use serde::Deserialize;
use serde::de::IntoDeserializer;
use uuid::Uuid;
use crate::types::network::*;
use crate::validation::error::{ValidationError, ValidationResult};


pub fn parse_and_validate_network_name(network_name: &str) -> ValidationResult<String> {
    if network_name.is_empty() {
        return Err(ValidationError::EmptyNetworkName());
    }
    Ok(network_name.to_string())
}

pub fn parse_and_validate_ipv4_subnet(subnet: &str) -> ValidationResult<Ipv4Net> {
    subnet.parse().map_err(|_| ValidationError::NotCIDR())
}

pub fn parse_and_validate_peer_id(peer_id: &str) -> ValidationResult<Uuid> {
    peer_id.parse().map_err(|_| ValidationError::InvalidUuid())
}

// Network.Peer Fields

pub fn parse_and_validate_peer_name(name: &str) -> ValidationResult<String> {
    if name.is_empty() {
        return Err(ValidationError::EmptyPeerName());
    }
    Ok(name.to_string())
}

pub fn parse_and_validate_peer_address(address: &str, network: &Network) -> ValidationResult<Ipv4Addr> {
    let address_ipv4 = address.parse().map_err(|_| ValidationError::NotIPv4Address())?;
    validate_peer_address(&address_ipv4, network)
}

pub fn validate_peer_address(address_ipv4: &Ipv4Addr, network: &Network) -> ValidationResult<Ipv4Addr> {
    if !network.subnet.contains(address_ipv4) {
        return Err(ValidationError::AddressNotInSubnet());
    }
    if *address_ipv4 == network.subnet.network() {
        return Err(ValidationError::AddressIsSubnetNetwork());
    }
    if *address_ipv4 == network.subnet.broadcast() {
        return Err(ValidationError::AddressIsSubnetBroadcast());
    }
    if let Some((peer_id, peer)) = network.peers.iter().find(|(_, p)| p.address == *address_ipv4) {
        return Err(ValidationError::AddressIsTaken(*peer_id, peer.name.clone()));
    }
    if network.reservations.get(address_ipv4).is_some_and(|res| Utc::now() < res.valid_until) {
        return Err(ValidationError::AddressIsReserved());
    }
    Ok(*address_ipv4)
}

pub fn parse_and_validate_peer_endpoint(endpoint_address: &str) -> ValidationResult<EndpointAddress> {
    if endpoint_address.is_empty() {
        return Ok(EndpointAddress::None);
    }

    // Split by last ':' to separate address and port
    let (address_str, port_str) = endpoint_address.rsplit_once(':')
        .ok_or(ValidationError::InvalidEndpoint())?;

    let port = port_str.parse::<u16>()
        .map_err(|_| ValidationError::InvalidEndpointPort())?;

    // Try parsing as IPv4 first, then fall back to the hostname
    if let Ok(ipv4) = address_str.parse::<Ipv4Addr>() {
        return Ok(EndpointAddress::Ipv4AndPort(Ipv4AndPort{ ipv4, port }));
    } else if hostname_validator::is_valid(address_str) {
        return Ok(EndpointAddress::HostnameAndPort(HostnameAndPort{ hostname: address_str.to_string(), port }));
    }

    Err(ValidationError::InvalidEndpoint())
}

pub fn validate_peer_endpoint(endpoint: &Endpoint) -> ValidationResult<Endpoint> {
    if let EndpointAddress::HostnameAndPort(h) = &endpoint.address
        && !hostname_validator::is_valid(&h.hostname) {
            return Err(ValidationError::InvalidEndpoint());
        }
    if endpoint.enabled && endpoint.address == EndpointAddress::None {
        return Err(ValidationError::EmptyEndpoint());
    }

    Ok(endpoint.clone())
}

pub fn parse_and_validate_peer_kind(kind: &str) -> ValidationResult<String> {
    // no validation
    Ok(kind.to_string())
}

pub fn parse_and_validate_peer_icon_src(src: &str) -> ValidationResult<String> {
    if src.is_empty() {
        return Err(ValidationError::EmptyIcon());
    }
    Ok(src.to_string())
}

pub fn validate_peer_icon(icon: &Icon) -> ValidationResult<Icon> {
    if icon.enabled {
        parse_and_validate_peer_icon_src(icon.src.as_str())?;
    }
    Ok(icon.clone())
}

pub fn parse_and_validate_peer_dns_addresses(dns: &str) -> ValidationResult<Vec<Ipv4Addr>> {
    let addresses = dns.split(',')
        .map(|address| address.trim().parse().map_err(|_| ValidationError::NotIPv4Address()))
        .collect::<ValidationResult<Vec<_>>>()?;

    Ok(addresses)
}

pub fn validate_peer_dns(dns: &Dns) -> ValidationResult<Dns> {
    if dns.enabled && dns.addresses.is_empty() {
        return Err(ValidationError::EmptyDns());
    }

    Ok(dns.clone())
}

pub fn parse_and_validate_peer_mtu_value(mtu_value: &str) -> ValidationResult<u16> {
    let mtu_value_u16 = mtu_value.parse::<u16>().map_err(|_| ValidationError::InvalidMtu())?;

    validate_peer_mtu_value(mtu_value_u16)
}

pub fn validate_peer_mtu_value(mtu_value: u16) -> ValidationResult<u16> {
    if mtu_value == 0 || mtu_value > 10000 {
        return Err(ValidationError::InvalidMtu());
    }

    Ok(mtu_value)
}

pub fn validate_peer_mtu(mtu: &Mtu) -> ValidationResult<Mtu> {
    if mtu.enabled {
        validate_peer_mtu_value(mtu.value)?;
    }

    Ok(mtu.clone())
}

pub fn parse_and_validate_peer_script(script: &str) -> ValidationResult<String> {
    if !script.trim().ends_with(';') {
        return Err(ValidationError::ScriptMissingSemicolon());
    }
    Ok(script.to_string())
}

pub fn validate_peer_script(script: &Script) -> ValidationResult<Script> {
    if script.enabled {
        parse_and_validate_peer_script(&script.script)?;
    }
    Ok(script.clone())
}

pub fn validate_peer_scripts(script: &[Script]) -> ValidationResult<Vec<Script>> {
    for (i, script) in script.iter().enumerate() {
        validate_peer_script(script).map_err(|_| {
            ValidationError::ScriptMissingSemicolonAt(i)
        })?;
    }
    Ok(script.to_owned())
}

pub fn parse_and_validate_wg_key(key: &str) -> ValidationResult<WireGuardKey> {
    WireGuardKey::deserialize(key.into_deserializer())
        .map_err(|_: serde::de::value::Error| ValidationError::NotWireGuardKey())
}

// Network.Connection Fields

pub fn parse_and_validate_conn_persistent_keepalive_period(persistent_keepalive_period: &str) -> ValidationResult<u16> {
    let period = persistent_keepalive_period.parse::<u16>()
        .map_err(|_| ValidationError::InvalidPersistentKeepalivePeriod())?;
    validate_conn_persistent_keepalive_period(period)
}

pub fn validate_conn_persistent_keepalive_period(persistent_keepalive_period: u16) -> ValidationResult<u16> {
    if persistent_keepalive_period == 0 {
        return Err(ValidationError::InvalidPersistentKeepalivePeriod());
    }
    Ok(persistent_keepalive_period)
}

pub fn validate_conn_persistent_keepalive(persistent_keepalive: &PersistentKeepalive) -> ValidationResult<PersistentKeepalive> {
    if persistent_keepalive.enabled {
        validate_conn_persistent_keepalive_period(persistent_keepalive.period)?;
    }

    Ok(persistent_keepalive.clone())
}

pub fn parse_and_validate_conn_allowed_ips(allowed_ips: &str) -> ValidationResult<AllowedIPs> {
    let ips = allowed_ips.split(',')
        .map(|cidr| cidr.trim().parse::<Ipv4Net>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ValidationError::InvalidAllowedIPs())?;

    Ok(ips)
}

