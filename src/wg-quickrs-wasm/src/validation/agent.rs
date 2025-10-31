#![cfg(not(target_arch = "wasm32"))]
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use crate::validation::error::{ValidationError, ValidationResult};
use crate::validation::helpers;


pub fn parse_and_validate_ipv4_address(address: &str) -> ValidationResult<Ipv4Addr> {
    address.parse().map_err(|_| ValidationError::NotIPv4Address())
}

pub fn parse_and_validate_port(port: &str) -> ValidationResult<u16> {
    port.parse().map_err(|_| ValidationError::NotPortNumber())
}

pub fn parse_and_validate_tls_file(config_folder: &Path, tls_file: &str) -> ValidationResult<PathBuf> {
    let tls_file_path = PathBuf::from(tls_file);
    validate_tls_file(config_folder, &tls_file_path)
}

pub fn validate_tls_file(config_folder: &Path, tls_file: &Path) -> ValidationResult<PathBuf> {
    let tls_file_path = config_folder.join(tls_file);

    if !tls_file_path.exists() {
        return Err(ValidationError::TlsFileNotFound());
    }

    if !tls_file_path.is_file() {
        return Err(ValidationError::TlsFileNotAFile());
    }

    Ok(tls_file_path)
}

pub fn parse_and_validate_fw_gateway(fw_gateway: &str) -> ValidationResult<String> {
    let interfaces = helpers::get_interfaces();

    // Try to find interface by name
    if let Some(iface) = interfaces.iter().find(|iface| iface.name == fw_gateway) {
        return Ok(iface.name.clone());
    }

    // If not found, prepare an error with available options
    let available: Vec<String> = interfaces
        .iter()
        .map(|iface| format!("{} ({})", iface.name, iface.ip()))
        .collect();

    Err(ValidationError::InterfaceNotFound(fw_gateway.to_string(), format!("[{}]", available.join(", "))))
}

pub fn parse_and_validate_fw_utility(fw_utility: &str) -> ValidationResult<PathBuf> {
    let fw_utility_path = PathBuf::from(fw_utility);
    validate_fw_utility(&fw_utility_path)
}

pub fn validate_fw_utility(fw_utility: &PathBuf) -> ValidationResult<PathBuf> {
    // Check if a path exists and is a file
    if fw_utility.exists() && fw_utility.is_file() {
        return Ok(fw_utility.to_path_buf());
    }

    // Not found - create a helpful error message
    let options = helpers::firewall_utility_options();
    Err(ValidationError::FirewallUtilityNotFound(
        fw_utility.display().to_string(),
        options.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ")
    ))
}

