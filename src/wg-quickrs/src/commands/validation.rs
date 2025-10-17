use crate::WG_QUICKRS_CONFIG_FOLDER;
use wg_quickrs_wasm::validation::{check_field_enabled_value, check_field_str, is_cidr, CheckResult};
use std::path::Path;
use wg_quickrs_wasm::types::EnabledValue;

pub fn check_field_str_agent(field_name: &str, field_variable: &str) -> CheckResult {
    let mut ret = CheckResult {
        status: false,
        msg: String::new(),
    };

    match field_name {
        "identifier" => {
            ret.status = !field_variable.is_empty();
            if !ret.status {
                ret.msg = "identifier cannot be empty".into();
            }
            ret
        }
        "subnet" => {
            ret.status = is_cidr(field_variable);
            if !ret.status {
                ret.msg = "subnet is not in CIDR format".into();
            }
            ret
        }
        "port" => {
            ret.status = !field_variable.parse::<u16>().is_err();
            if !ret.status {
                ret.msg = "Port is invalid".into();
            }
            ret
        }
        "firewall-gateway" => {
            let mut gateways: Vec<String> = Vec::new();
            for iface in crate::commands::init::get_interfaces() {
                if iface.name == field_variable {
                    ret.status = true;
                }
                gateways.push(format!("{} ({})", iface.name, iface.ip()));
            }
            if !ret.status {
                ret.msg = format!(
                    "Gateway not found: {} (possible options: {})",
                    field_variable,
                    gateways.join(", ")
                );
            }
            ret
        }
        _ => check_field_str(field_name, field_variable),
    }
}

pub fn check_field_enabled_value_agent(field_name: &str, field_variable: &EnabledValue) -> CheckResult {
    check_field_enabled_value(field_name, field_variable)
}

pub fn check_field_path_agent(field_name: &str, field_variable: &Path) -> CheckResult {
    let mut ret = CheckResult {
        status: false,
        msg: String::new(),
    };

    match field_name {
        "path" => {
            let config_folder = WG_QUICKRS_CONFIG_FOLDER.get().unwrap();
            let tls_file_path = config_folder.join(field_variable);
            ret.status = tls_file_path.exists() && tls_file_path.is_file();
            if !ret.status {
                ret.msg = format!("File not found: {}", tls_file_path.display());
            }
            ret
        }
        "firewall-utility" => {
            let bin_path = field_variable;
            ret.status = bin_path.exists() && bin_path.is_file();
            if !ret.status {
                ret.msg = format!(
                    "Binary not found at path: {} (possible options: {})",
                    bin_path.display(),
                    crate::commands::init::firewall_utility_options().join(", ")
                );
            }
            ret
        }
        _ => {
            ret.status = false;
            ret.msg = "field doesn't exist".into();
            ret
        }
    }
}
