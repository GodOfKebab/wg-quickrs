use ipnet::Ipv4Net;
use regex_lite::Regex;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddrV4};
use base64::Engine;
use crate::types::EnabledValue;
use base64::engine::general_purpose::STANDARD;


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

pub fn check_field_str(field_name: &str, field_variable: &str) -> CheckResult {
    let mut ret = CheckResult {
        status: false,
        msg: String::new(),
    };

    match field_name {
        // UUID v4 check
        "peerId" => {
            let re_uuid = Regex::new(
                r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$",
            )
                .unwrap();
            ret.status = re_uuid.is_match(field_variable);
            if !ret.status {
                ret.msg = "peerId needs to follow uuid4 standards".into();
            }
        }

        "name" => {
            ret.status = !field_variable.is_empty();
            if !ret.status {
                ret.msg = "name cannot be empty".into();
            }
        }

        // TODO: check subnet
        // TODO: check to see if a duplicate exists
        "address" => {
            ret.status = is_ipv4(field_variable);
            if !ret.status {
                ret.msg = "address is not IPv4".into();
            }
        }

        "kind" => {
            ret.status = true;
        }

        "public_key" | "private_key" | "pre_shared_key" => {
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
            if field_variable.enabled {
                let re = Regex::new(r"^.*;\s*$").unwrap();
                if !re.is_match(&field_variable.value) {
                    ret.status = false;
                }
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

