#![cfg(target_os = "macos")]
use std::collections::HashMap;
use std::fs;
use std::net::Ipv4Addr;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::time::{Duration, SystemTime};
use log::{log_enabled, Level};
use regex::Regex;
use wg_quickrs_lib::types::network::{Dns, Mtu};
use crate::helpers::shell_cmd;
use crate::wireguard::wg_quick;
use crate::wireguard::wg_quick::{DnsManager, EndpointRouter, TunnelError, TunnelResult};


pub fn interface_exists(interface: &str) -> TunnelResult<Option<String>> {
    let name_path = format!("/var/run/wireguard/{}.name", interface);
    if !std::path::PathBuf::from(&name_path).exists() {
        return Ok(None);
    }

    let iface = fs::read_to_string(&name_path)?.trim().to_string();
    if iface.is_empty() {
        return Ok(None);
    }

    let sock_path = format!("/var/run/wireguard/{}.sock", iface);
    match fs::metadata(&sock_path) {
        Ok(m) => {
            if !m.file_type().is_socket() {
                return Ok(None);
            }
        }
        Err(_) => {
            return Ok(None);
        }
    }

    // Get modification times (with fallback values)
    let sock_mtime = fs::metadata(&sock_path)
        .and_then(|m| m.modified())
        .map(|t| t.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs())
        .unwrap_or(200);

    let name_mtime = fs::metadata(&name_path)
        .and_then(|m| m.modified())
        .map(|t| t.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs())
        .unwrap_or(100);

    let diff = (sock_mtime as i64) - (name_mtime as i64);

    if diff.abs() >= 2 {
        return Ok(None);
    }

    Ok(Some(iface))
}

pub fn add_interface(interface: &str, userspace: bool, userspace_binary: &str) -> TunnelResult<String> {
    if !userspace {
        return Err(TunnelError::NoWireGuardKernelModule());
    }

    fs::create_dir_all("/var/run/wireguard/")?;
    let name_file = format!("/var/run/wireguard/{}.name", interface);

    let log_level = if log_enabled!(Level::Debug) { "debug" } else { "error" };
    let name_file_cloned = name_file.clone();
    let userspace_binary_cloned = userspace_binary.to_string();
    std::thread::spawn(move || unsafe {
        match Command::new(&userspace_binary_cloned)
            .args(["--foreground", "utun"])
            .env("WG_TUN_NAME_FILE", name_file_cloned)
            .env("LOG_LEVEL", log_level)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .pre_exec(|| {
                // Create a new session -> child will NOT be in parent’s process group
                libc::setsid(); // unsafe function
                Ok(())
            }).output() {
            Ok(output) => {
                if !output.status.success() {
                    log::error!("[!] {} failed: {}", userspace_binary_cloned, String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                log::error!("[!] {} failed: {}", userspace_binary_cloned, e);
            }
        };

        log::debug!("[+] {} exit", userspace_binary_cloned);
    });

    std::thread::sleep(Duration::from_millis(500)); // TODO: replace with a better solution

    let iface = fs::read_to_string(&name_file)?.trim().to_string();
    Ok(iface)
}

pub fn del_interface(_wg: &str, iface: &str, interface: &str, _dns_manager: &mut DnsManager, _endpoint_router: &mut EndpointRouter) -> TunnelResult<()> {
    let sock_file = format!("/var/run/wireguard/{}.sock", iface);
    let _ = fs::remove_file(sock_file);

    let name_file = format!("/var/run/wireguard/{}.name", interface);
    let _ = fs::remove_file(name_file);

    Ok(())
}

pub fn add_address(iface: &str, addr: &str, is_ipv6: bool) -> TunnelResult<()> {
    if is_ipv6 {
        shell_cmd(&["ifconfig", iface, "inet6", addr, "alias"])?;
    } else {
        let ip = addr.split('/').next().unwrap();
        shell_cmd(&["ifconfig", iface, "inet", addr, ip, "alias"])?;
    }

    Ok(())
}

pub fn set_mtu_and_up(_wg: &str, iface: &str, mtu: &Mtu) -> TunnelResult<()> {
    set_mtu(iface, mtu)?;
    shell_cmd(&["ifconfig", iface, "up"])?;

    Ok(())
}

pub fn set_mtu(iface: &str, mtu: &Mtu) -> TunnelResult<()> {
    let mtu_val = if mtu.enabled {
        mtu.value
    } else {
        // Find default interface
        let netstat_output = shell_cmd(&["netstat", "-nr", "-f", "inet"])?;

        let output_str = String::from_utf8_lossy(&netstat_output.stdout);
        let default_if = output_str
            .lines()
            .find(|line| line.starts_with("default"))
            .and_then(|line| line.split_whitespace().nth(5));

        // Get MTU from default interface
        let mut mtu_sys = 1500u16; // fallback
        if let Some(default_if) = default_if
            && let Some(detected_mtu) = get_interface_mtu(default_if)? {
                mtu_sys = detected_mtu;
            }

        // Subtract WireGuard overhead
        mtu_sys = mtu_sys.saturating_sub(80);
        mtu_sys
    };

    // Only set if different from current
    if let Some(current_mtu) = get_interface_mtu(iface)?
        && mtu_val == current_mtu {
            return Ok(());
        }
    shell_cmd(&["ifconfig", iface, "mtu", &mtu_val.to_string()])?;

    Ok(())
}

fn get_interface_mtu(iface: &str) -> TunnelResult<Option<u16>> {
    let output = shell_cmd(&["ifconfig", iface])?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let re = Regex::new(r"mtu\s+(\d+)").unwrap();

    Ok(re.captures(&output_str)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse().ok()))
}

fn collect_services(dns_manager: &mut DnsManager) -> TunnelResult<()> {
    let output = shell_cmd(&["networksetup", "-listallnetworkservices"])?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut found_services = HashMap::new();

    for (i, line) in output_str.lines().enumerate() {
        if i == 0 { continue; } // Skip header

        let service = line.trim_start_matches('*');
        found_services.insert(service.to_string(), true);

        // Skip if already captured
        if dns_manager.service_dns.contains_key(service) {
            continue;
        }

        // Get DNS servers
        if let Ok(dns) = get_dns_servers(service) {
            dns_manager.service_dns.insert(service.to_string(), dns);
        }

        // Get search domains
        if let Ok(search) = get_search_domains(service) {
            dns_manager.service_dns_search.insert(service.to_string(), search);
        }
    }

    // Remove services that no longer exist
    dns_manager.service_dns.retain(|k, _| found_services.contains_key(k));
    dns_manager.service_dns_search.retain(|k, _| found_services.contains_key(k));

    Ok(())
}

fn get_dns_servers(service: &str) -> TunnelResult<String> {
    let output = shell_cmd(&["networksetup", "-getdnsservers", service])?;
    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // If contains spaces, it's an error message
    Ok(if result.contains(' ') {
        "Empty".to_string()
    } else {
        result
    })
}

fn get_search_domains(service: &str) -> TunnelResult<String> {
    let output = shell_cmd(&["networksetup", "-getsearchdomains", service])?;
    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // If contains spaces, it's an error message
    Ok(if result.contains(' ') {
        "Empty".to_string()
    } else {
        result
    })
}

pub fn set_dns(dns_servers: &[Ipv4Addr], _interface: &str, dns_manager: &mut DnsManager) -> TunnelResult<()> {
    collect_services(dns_manager)?;

    for service in dns_manager.service_dns.keys() {
        // Set DNS servers
        let mut set_dns_cmd = vec!["networksetup", "-setdnsservers", service];
        let dns_strings: Vec<String> = dns_servers.iter().map(|s| s.to_string()).collect();
        set_dns_cmd.extend(dns_strings.iter().map(|s| s.as_str()));
        let _ = shell_cmd(&set_dns_cmd)?;

        // Set search domains - always set search domains to Empty since DNS_SEARCH is empty
        let _ = shell_cmd(&["networksetup", "-setsearchdomains", service, "Empty"])?;
    }

    Ok(())
}

pub fn del_dns(_interface: &str, dns_manager: &mut DnsManager) -> TunnelResult<()> {
    for (service, original_dns) in &dns_manager.service_dns {
        // Restore DNS (ignore errors)
        let mut set_dns_cmd = vec!["networksetup", "-setdnsservers", service];
        set_dns_cmd.extend(original_dns.split_whitespace());
        let _ = shell_cmd(&set_dns_cmd)?;

        // Restore search domains (ignore errors)
        if let Some(search) = dns_manager.service_dns_search.get(service) {
            let mut set_dns_search_cmd = vec!["networksetup", "-setsearchdomains", service];
            set_dns_search_cmd.extend(search.split_whitespace());
            let _ = shell_cmd(&set_dns_search_cmd)?;
        }
    }
    *dns_manager = Default::default(); // reset DNS manager

    Ok(())
}

pub fn add_route(_wg: &str, iface: &str, _interface_name: &str, cidr: &str, endpoint_router: &mut EndpointRouter) -> TunnelResult<()> {
    let is_default = cidr.ends_with("/0");
    let is_ipv6 = cidr.contains(':');

    if is_default {
        if is_ipv6 {
            shell_cmd(&["route", "-q", "-n", "add", "-inet6", "::/1", "-interface", iface])?;
            shell_cmd(&["route", "-q", "-n", "add", "-inet6", "8000::/1", "-interface", iface])?;
            endpoint_router.auto_route6 = true;
        } else {
            shell_cmd(&["route", "-q", "-n", "add", "-inet", "0.0.0.0/1", "-interface", iface])?;
            shell_cmd(&["route", "-q", "-n", "add", "-inet", "128.0.0.0/1", "-interface", iface])?;
            endpoint_router.auto_route4 = true;
        }
    } else {
        let family = if is_ipv6 { "-inet6" } else { "-inet" };

        // Check if a route already exists through this interface
        let route_exists = match shell_cmd(&["route", "-n", "get", family, cidr]) {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                output_str.contains(&format!("interface: {}\n", iface))
            }
            Err(_) => false
        };
        if !route_exists {
            let _ = shell_cmd(&["route", "-q", "-n", "add", family, cidr, "-interface", iface]);
        }
    }

    Ok(())
}

pub fn set_endpoint_direct_route(wg: &str, iface: &str, endpoint_router: &mut EndpointRouter) -> TunnelResult<()> {
    let mut old_endpoints = endpoint_router.endpoints.clone();
    let old_gateway4 = endpoint_router.gateway4.clone();
    let old_gateway6 = endpoint_router.gateway6.clone();

    endpoint_router.gateway4 = get_default_gateway(false).ok();
    endpoint_router.gateway6 = get_default_gateway(true).ok();
    endpoint_router.endpoints = wg_quick::get_endpoints(wg, iface);

    // Check if gateways changed
    let remove_all_old = old_gateway4 != endpoint_router.gateway4 || old_gateway6 != endpoint_router.gateway6;

    // Build list of endpoints to remove
    if remove_all_old {
        // Add any new endpoints to a removal list to ensure a clean slate
        for ep in &endpoint_router.endpoints {
            if !old_endpoints.contains(ep) {
                old_endpoints.push(ep.clone());
            }
        }
    }

    // Remove old routes
    for endpoint in &old_endpoints {
        if !remove_all_old && endpoint_router.endpoints.contains(endpoint) {
            continue; // Keep existing routes that are still needed
        }

        if endpoint.contains(':') && endpoint_router.auto_route6 {
            let _ = shell_cmd(&["route", "-q", "-n", "delete", "-inet6", endpoint]);
        } else if endpoint_router.auto_route4 {
            let _ = shell_cmd(&["route", "-q", "-n", "delete", "-inet", endpoint]);
        }
    }

    // Add new routes
    let mut added = Vec::new();
    for endpoint in &endpoint_router.endpoints {
        // Skip if already existed and wasn't removed
        if !remove_all_old && old_endpoints.contains(endpoint) {
            added.push(endpoint.clone());
            continue;
        }

        if endpoint.contains(':') && endpoint_router.auto_route6 {
            // IPv6 endpoint
            if let Some(gw) = &endpoint_router.gateway6 {
                let _ = shell_cmd(&["route", "-q", "-n", "add", "-inet6", endpoint, "-gateway", gw]);
            } else {
                // No IPv6 gateway - add a blackhole route to prevent routing loop
                let _ = shell_cmd(&["route", "-q", "-n", "add", "-inet6", endpoint, "::1", "-blackhole"]);
            }
            added.push(endpoint.clone());
        } else if endpoint_router.auto_route4 {
            // IPv4 endpoint
            if let Some(gw) = &endpoint_router.gateway4 {
                let _ = shell_cmd(&["route", "-q", "-n", "add", "-inet", endpoint, "-gateway", gw]);
            } else {
                // No IPv4 gateway - add blackhole route
                let _ = shell_cmd(&["route", "-q", "-n", "add", "-inet", endpoint, "127.0.0.1", "-blackhole"]);
            }
            added.push(endpoint.clone());
        }
    }

    endpoint_router.endpoints = added;
    Ok(())
}
fn get_default_gateway(ipv6: bool) -> TunnelResult<String> {
    let family = if ipv6 { "inet6" } else { "inet" };

    let output = shell_cmd(&["netstat", "-nr", "-f", family])?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 1 && parts[0] == "default" && !parts[1].starts_with("link#") {
            return Ok(parts[1].to_string());
        }
    }
    Err(TunnelError::DefaultGatewayNotFound())
}

pub fn start_monitor_daemon(wg: &str, iface: &str, interface_name: &str, dns: &Dns, mtu: &Mtu, endpoint_router: &EndpointRouter, dns_manager: &DnsManager) -> TunnelResult<()> {
    let wg_clone = wg.to_string();
    let iface_clone = iface.to_string();
    let interface_name_clone = interface_name.to_string();
    let dns_clone = dns.clone();
    let mtu_clone = mtu.clone();
    let endpoint_router_clone = endpoint_router.clone();
    let mut dns_manager_clone = dns_manager.clone();
    std::thread::spawn(move || {
        let result = monitor_daemon_worker(
            wg_clone,
            iface_clone.clone(),
            interface_name_clone,
            dns_clone,
            mtu_clone,
            endpoint_router_clone,
            dns_manager_clone.clone(),
        );

        if let Err(e) = result {
            log::warn!("[!] Monitor daemon error: {}", e);
        }
        if let Err(e) = del_routes(&iface_clone) {
            log::warn!("[!] Monitor daemon error while deleting routes: {}", e);
        }
        if let Err(e) = del_dns(&iface_clone, &mut dns_manager_clone) {
            log::warn!("[!] Monitor daemon error while deleting DNS: {}", e);
        }
        log::debug!("[+] Stopped route monitor");
    });

    log::debug!("[+] Backgrounding route monitor");
    Ok(())
}

fn monitor_daemon_worker(
    wg: String,
    real_iface: String,
    _interface: String,
    dns: Dns,
    mtu: Mtu,
    endpoint_router: EndpointRouter,
    dns_manager: DnsManager,
) -> TunnelResult<()> {
    use std::process::Stdio;
    use std::io::{BufRead, BufReader};

    let dns_servers = if dns.enabled {
        dns.addresses
    } else {
        Vec::new()
    };

    let mut route_monitor = Command::new("route")
        .args(["-n", "monitor"])
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = route_monitor.stdout.take()
        .ok_or_else(|| TunnelError::CommandFailed("Failed to capture route monitor output".to_string()))?;

    let reader = BufReader::new(stdout);

    let mut endpoint_router_clone = endpoint_router.clone();
    let mut dns_manager_clone = dns_manager.clone();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if !line.starts_with("RTM_") {
            continue;
        }

        let iface_check = shell_cmd(&["ifconfig", &real_iface]);

        if iface_check.is_err() || !iface_check?.status.success() {
            log::warn!("[!] Interface {} no longer exists, stopping monitor", &real_iface);
            break;
        }

        if (endpoint_router.auto_route4 || endpoint_router.auto_route6)
            && let Err(e) = set_endpoint_direct_route(&wg, &real_iface, &mut endpoint_router_clone) {
                log::warn!("[!] Failed to reapply endpoint routes: {}", e);
            }

        if mtu.enabled
            && let Err(e) = set_mtu(&real_iface, &mtu) {
                log::warn!("[!] Failed to reapply MTU: {}", e);
            }

        // Reapply DNS with debouncing
        if !dns_servers.is_empty() {
            if let Err(e) = set_dns(&dns_servers, &real_iface, &mut dns_manager_clone) {
                log::warn!("[!] Failed to reapply DNS: {}", e);
            }
            // Debounce: reapply after 2 seconds
            let dns_clone = dns_servers.clone();
            let real_iface_clone = real_iface.clone();
            let mut dns_manager_clone = dns_manager_clone.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(2));
                if let Err(e) = set_dns(&dns_clone, &real_iface_clone, &mut dns_manager_clone) {
                    log::warn!("[!] Failed to reapply DNS: {}", e);
                }
            });
        }
    }

    log::debug!("[+] Stopping route monitor");
    let _ = route_monitor.kill();
    log::debug!("[+] Stopped route monitor");

    Ok(())
}

pub fn del_routes(iface: &str) -> TunnelResult<()> {
    let output = shell_cmd(&["netstat", "-nr", "-f", "inet"])?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 5 && parts[5] == iface {
            let dest = parts[0];
            let _ = shell_cmd(&["route", "-q", "-n", "delete", "-inet", dest]);
        }
    }

    let output = shell_cmd(&["netstat", "-nr", "-f", "inet6"])?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 3 && parts[3] == iface {
            let dest = parts[0];
            let _ = shell_cmd(&["route", "-q", "-n", "delete", "-inet6", dest]);
        }
    }

    Ok(())
}
