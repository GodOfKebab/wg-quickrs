use crate::WG_QUICKRS_CONFIG_FOLDER;
use rust_wasm::validation::{check_field, is_cidr, CheckResult, FieldValue};
use std::path::PathBuf;

pub fn check_field_agent(field_name: &str, field_variable: &FieldValue) -> CheckResult {
    let mut ret = CheckResult {
        status: false,
        msg: String::new(),
    };

    match field_name {
        "identifier" => {
            ret.status = !field_variable.str.is_empty();
            if !ret.status {
                ret.msg = "identifier cannot be empty".into();
            }
            ret
        }
        "subnet" => {
            ret.status = is_cidr(&field_variable.str);
            if !ret.status {
                ret.msg = "subnet is not in CIDR format".into();
            }
            ret
        }
        "port" => {
            ret.status = true;
            if let Ok(v) = field_variable.enabled_value.value.parse::<i32>() {
                ret.status = v > 0 && v < 65536;
            } else {
                ret.status = false; // not a number
            }
            if !ret.status {
                ret.msg = "Port is invalid".into();
            }
            ret
        }
        "path" => {
            let config_folder = WG_QUICKRS_CONFIG_FOLDER.get().unwrap();
            let tls_file_path = config_folder.join(field_variable.str.clone());
            ret.status = tls_file_path.exists() && tls_file_path.is_file();
            if !ret.status {
                ret.msg = format!("File not found: {}", tls_file_path.display());
            }
            ret
        }
        "gateway" => {
            let mut gateways: Vec<String> = Vec::new();
            for iface in crate::commands::init::get_interfaces() {
                if iface.name == field_variable.str {
                    ret.status = true;
                }
                gateways.push(format!("{} ({})", iface.name, iface.ip()));
            }
            if !ret.status {
                ret.msg = format!(
                    "Gateway not found: {} (possible options: {})",
                    field_variable.str.clone(),
                    gateways.join(", ")
                );
            }
            ret
        }
        "firewall" => {
            let bin_path = PathBuf::from(field_variable.enabled_value.value.clone());
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
        _ => check_field(field_name, field_variable),
    }
}
