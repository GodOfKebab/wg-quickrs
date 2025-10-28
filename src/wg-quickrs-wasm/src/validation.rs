use ipnet::Ipv4Net;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddrV4};
use base64::Engine;
use crate::types::{EnabledValue, Network};
use base64::engine::general_purpose::STANDARD;
use chrono::{Duration, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckResult {
    pub status: bool,
    pub msg: String,
}

// Helper: plain IPv4
fn is_ipv4(s: &str) -> bool {
    s.parse::<Ipv4Addr>().is_ok()
}

// Helper: IPv4 + port
fn is_ipv4_with_port(s: &str) -> bool {
    s.parse::<SocketAddrV4>().is_ok()
}

// Helper: CIDR IPv4 network
pub fn is_cidr(s: &str) -> bool {
    s.parse::<Ipv4Net>().is_ok()
}

// Helper: FQDN + port
pub fn is_fqdn_with_port(s: &str) -> bool {
    // Split on the last colon to separate the hostname and port
    match s.rsplit_once(':') {
        Some((hostname, port_str)) => {
            if port_str.parse::<u16>().is_err() {
                return false;
            }
            if hostname.chars().all(|c| c.is_ascii_digit() || c == '.') {
                return false;
            }
            // Validate hostname with validate-hostname crate
            hostname_validator::is_valid(hostname)
        }
        None => false, // no colon, no port
    }
}

pub fn check_internal_address(address: &str, network: &Network) -> CheckResult {
    let Ok(address_ipv4) = address.parse::<Ipv4Addr>() else {
        return CheckResult { status: false, msg: "address is not IPv4".into() };
    };
    let Ok(subnet_ipv4) = network.subnet.parse::<Ipv4Net>() else {
        return CheckResult { status: false, msg: "can't check address because network subnet is not in CIDR format".into() };
    };
    if !subnet_ipv4.contains(&address_ipv4) {
        return CheckResult { status: false, msg: "address is not in the network subnet".into() };
    }
    if address_ipv4 == subnet_ipv4.network() {
        return CheckResult { status: false, msg: "address is the network address and cannot be assigned".into() };
    }
    if address_ipv4 == subnet_ipv4.broadcast() {
        return CheckResult { status: false, msg: "address is the broadcast address and cannot be assigned".into() };
    }
    if let Some((peer_id, peer)) = network.peers.iter().find(|(_, peer)| peer.address == address) {
        return CheckResult { status: false, msg: format!("address is already taken by {}({})", peer.name, peer_id) };
    }
    if let Some(reservation_data) = network.reservations.get(address) {
        if Utc::now().signed_duration_since(&reservation_data.valid_until) < Duration::zero() {
            return CheckResult { status: false, msg: "address is reserved for another peer".into() }
        }
    }
    CheckResult { status: true, msg: "".into() }
}

pub fn check_field_str(field_name: &str, field_variable: &str) -> CheckResult {
    let mut ret = CheckResult {
        status: false,
        msg: String::new(),
    };

    match field_name {
        "name" => {
            ret.status = !field_variable.is_empty();
            if !ret.status {
                ret.msg = "name cannot be empty".into();
            }
        }

        "generic-address" => {
            ret.status = is_ipv4(field_variable);
            if !ret.status {
                ret.msg = "address is not IPv4".into();
            }
        }

        "kind" => {
            ret.status = true;
        }

        "private_key" | "pre_shared_key" => {
            ret.status = match STANDARD.decode(field_variable) {
                Ok(bytes) => bytes.len() == 32,
                Err(_) => false
            };
            if !ret.status {
                ret.msg = format!("{field_name} is not base64 encoded with 32 bytes (got {bytes_len})", bytes_len = field_variable.len());
            }
        }

        "allowed_ips_a_to_b" | "allowed_ips_b_to_a" => {
            ret.status = field_variable
                .split(',')
                .all(|cidr| is_cidr(cidr.trim()));
            if !ret.status {
                ret.msg = "AllowedIPs is not in CIDR format".into();
            }
        }

        _ => {
            ret.status = false;
            ret.msg = "field doesn't exist".into();
        }
    }

    ret
}

pub fn check_field_enabled_value(field_name: &str, field_variable: &EnabledValue) -> CheckResult {
    let mut ret = CheckResult {
        status: false,
        msg: String::new(),
    };

    match field_name {
        "endpoint" => {
            if field_variable.enabled
                && !(is_ipv4_with_port(&field_variable.value)
                    || is_fqdn_with_port(&field_variable.value))
            {
                ret.status = false;
                ret.msg = "endpoint is not IPv4 nor an FQDN".into();
            } else {
                ret.status = true;
            }
        }

        "icon" => {
            ret.status = true;
            if field_variable.enabled {
                ret.status = !field_variable.value.is_empty();
            }
            if !ret.status {
                ret.msg = "icon cannot be empty".into();
            }

        }

        "dns" => {
            ret.status = true;
            if field_variable.enabled {
                // Allow multiple DNS servers, comma-separated
                ret.status = field_variable
                    .value
                    .split(',')
                    .all(|addr| is_ipv4(addr.trim()));
            }
            if !ret.status {
                ret.msg = "DNS is invalid".into();
            }
        }

        "mtu" => {
            ret.status = true;
            if field_variable.enabled {
                    if field_variable.value.parse::<u16>().is_err() {
                        ret.status = false;
                    } else if let Ok(mtu_val) = field_variable.value.parse::<u16>() {
                        ret.status = 0 < mtu_val && mtu_val < 10000;
                    }
                }
            if !ret.status {
                ret.msg = "MTU is invalid (1-9999)".into();
            }
        }

        "script" | "pre_up" | "post_up" | "pre_down" | "post_down" => {
            ret.status = true;
            if field_variable.enabled
                && !field_variable.value.ends_with(';') {
                    ret.status = false;
                }
            if !ret.status {
                ret.msg = "script needs to end with a semicolon".into();
            }
        }

        "persistent_keepalive" => {
            ret.status = true;
            if field_variable.enabled
                && field_variable.value.parse::<u16>().is_err() {
                    ret.status = false;
                }
            if !ret.status {
                ret.msg = "Persistent Keepalive is invalid".into();
            }
        }

        _ => {
            ret.status = false;
            ret.msg = "field doesn't exist".into();
        }
    }

    ret
}

