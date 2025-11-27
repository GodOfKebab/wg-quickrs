#![cfg(not(target_arch = "wasm32"))]
use std::net::Ipv4Addr;
use std::os::unix::fs::PermissionsExt;
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

    Ok(tls_file.to_path_buf())
}

pub fn parse_and_validate_wg_tool(wg_tool: &str) -> ValidationResult<PathBuf> {
    let wg_tool_path = PathBuf::from(wg_tool);
    validate_wg_tool(&wg_tool_path)
}

pub fn validate_wg_tool(wg_tool: &Path) -> ValidationResult<PathBuf> {
    let is_executable = wg_tool
        .metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false);

    if is_executable {
        Ok(wg_tool.to_path_buf())
    } else {
        Err(ValidationError::WgToolNotFound(
            wg_tool.display().to_string(),
            helpers::wg_tool_options().iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ")
        ))
    }
}

pub fn parse_and_validate_wg_userspace_binary(wg_userspace_binary: &str) -> ValidationResult<PathBuf> {
    let wg_userspace_binary_path = PathBuf::from(wg_userspace_binary);
    validate_wg_userspace_binary(&wg_userspace_binary_path)
}

pub fn validate_wg_userspace_binary(wg_userspace_binary: &Path) -> ValidationResult<PathBuf> {
    let is_executable = wg_userspace_binary
        .metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false);

    if is_executable {
        Ok(wg_userspace_binary.to_path_buf())
    } else {
        Err(ValidationError::WgUserspaceNotFound(
            wg_userspace_binary.display().to_string(),
            helpers::wg_userspace_options().iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ")
        ))
    }
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

pub fn validate_fw_utility(fw_utility: &Path) -> ValidationResult<PathBuf> {
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

