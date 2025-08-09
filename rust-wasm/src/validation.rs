use crate::types;
use ipnet::Ipv4Net;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddrV4};

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckResult {
    pub status: bool,
    pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValue {
    pub str: String,
    pub enabled_value: types::EnabledValue,
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
fn is_cidr(s: &str) -> bool {
    s.parse::<Ipv4Net>().is_ok()
}

// Helper: FQDN + port
fn is_fqdn_with_port(s: &str) -> bool {
    // Split on the last colon to separate the hostname and port
    match s.rsplit_once(':') {
        Some((hostname, port_str)) => {
            // port 0-65535 is valid: comparison is useless due to type limits
            if port_str.parse::<u16>().is_err() {
                return false;
            }
            // Validate hostname with validate-hostname crate
            hostname_validator::is_valid(hostname)
        }
        None => false, // no colon, no port
    }
}

pub fn check_field(field_name: &str, field_variable: &FieldValue) -> CheckResult {
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
            ret.status = re_uuid.is_match(&field_variable.str);
            if !ret.status {
                ret.msg = "peerId needs to follow uuid4 standards".into();
            }
        }

        "name" => {
            ret.status = !field_variable.str.is_empty();
            if !ret.status {
                ret.msg = "name cannot be empty".into();
            }
        }

        // TODO: check subnet
        // TODO: check to see if a duplicate exists
        "address" => {
            ret.status = is_ipv4(&field_variable.str);
            if !ret.status {
                ret.msg = "address is not IPv4".into();
            }
        }

        "endpoint" => {
            if field_variable.enabled_value.enabled
                && !(is_ipv4_with_port(&field_variable.enabled_value.value)
                || is_fqdn_with_port(&field_variable.enabled_value.value))
            {
                ret.status = false;
                ret.msg = "endpoint is not IPv4 nor an FQDN".into();
            } else {
                ret.status = true;
            }
        }

        "dns" => {
            ret.status = true;
            if field_variable.enabled_value.enabled {
                // Allow multiple DNS servers, comma-separated
                ret.status = field_variable
                    .enabled_value
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
            if let Ok(v) = field_variable.enabled_value.value.parse::<i32>() {
                ret.status = v > 0 && v < 65536;
            } else {
                ret.status = false; // not a number
            }
            if !ret.status {
                ret.msg = "MTU is invalid".into();
            }
        }

        "script" | "pre_up" | "post_up" | "pre_down" | "post_down" => {
            ret.status = true;
            if field_variable.enabled_value.enabled {
                let re = Regex::new(r"^.*;\s*$").unwrap();
                if !re.is_match(&field_variable.enabled_value.value) {
                    ret.status = false;
                }
            }
            if !ret.status {
                ret.msg = "script needs to end with a semicolon".into();
            }
        }

        // TODO: implement me
        "public_key" => {
            ret.status = !field_variable.str.is_empty();
            if !ret.status {
                ret.msg = "public_key cannot be empty".into();
            }
        }

        // TODO: implement me
        "private_key" => {
            ret.status = !field_variable.str.is_empty();
            if !ret.status {
                ret.msg = "private_key cannot be empty".into();
            }
        }

        // TODO: implement me
        "pre_shared_key" => {
            ret.status = !field_variable.str.is_empty();
            if !ret.status {
                ret.msg = "pre_shared_key cannot be empty".into();
            }
        }

        "allowed_ips_a_to_b" | "allowed_ips_b_to_a" => {
            ret.status = field_variable
                .str
                .split(',')
                .all(|cidr| is_cidr(cidr.trim()));
            if !ret.status {
                ret.msg = "AllowedIPs is not in CIDR format".into();
            }
        }

        "persistent_keepalive" => {
            ret.status = true;
            if let Ok(v) = field_variable.enabled_value.value.parse::<i32>() {
                ret.status = v > 0 && v < 65536;
            } else {
                ret.status = false; // not a number
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

#[macro_export]
macro_rules! validation_check_field_str {
    ($field:ident, $value:expr) => {
        check_field(stringify!($field), &FieldValue {
            str: $value.clone(),
            enabled_value: EnabledValue {enabled: false, value: String::new()},
        });
    }
}

#[macro_export]
macro_rules! validation_check_field_enabled_value {
    ($field:ident, $value:expr) => {
        check_field(stringify!($field), &FieldValue {
            str: String::new(),
            enabled_value: $value.clone(),
        });
    }
}
