use std::env;
use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use get_if_addrs::{get_if_addrs, Interface};

/// Get primary IP of the current machine
#[cfg(not(target_arch = "wasm32"))]
pub fn get_interfaces() -> Vec<Interface> {
    let mut interfaces: Vec<Interface> = Vec::new();
    for iface in get_if_addrs()
        .unwrap()
        .iter()
        .filter(|a| !a.is_loopback() && a.ip().is_ipv4())
    {
        interfaces.push(iface.clone());
    }
    interfaces
}

/// Find a command in the PATH environment variable.
fn find_in_path(cmd: &str) -> Option<PathBuf> {
    if let Ok(paths) = env::var("PATH") {
        for dir in env::split_paths(&paths) {
            let full_path = dir.join(cmd);
            if full_path.is_file() {
                return Some(full_path);
            }
        }
    }
    None
}

/// Get a list of firewall utilities available on the system.
pub fn firewall_utility_options() -> Vec<String> {
    let candidates = ["iptables", "pfctl"];
    let mut ret: Vec<String> = Vec::new();
    for prog in candidates {
        if let Some(path) = find_in_path(prog) {
            ret.push(path.to_str().unwrap().to_string());
        }
    }
    ret
}
