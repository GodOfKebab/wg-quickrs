#![cfg(target_os = "macos")]
use std::fs;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use wg_quickrs_wasm::types::EnabledValue;
use crate::wireguard::wg_quick;
use crate::wireguard::wg_quick::{TunnelError, TunnelResult};


pub fn interface_exists(interface: &str) -> TunnelResult<Option<String>> {
    let name_file = format!("/var/run/wireguard/{}.name", interface);
    if !std::path::PathBuf::from(&name_file).exists() {
        return Ok(None);
    }

    let iface = fs::read_to_string(&name_file)?.trim().to_string();
    let sock_file = format!("/var/run/wireguard/{}.sock", iface);

    if std::path::PathBuf::from(&sock_file).exists() {
        return Ok(Some(iface));
    }
    Ok(None)
}

pub fn add_interface(interface: &str) -> TunnelResult<String> {
    fs::create_dir_all("/var/run/wireguard/")?;

    let name_file = format!("/var/run/wireguard/{}.name", interface);

    let value = name_file.clone();
    std::thread::spawn(move || unsafe {
        match Command::new("wireguard-go")
            .args(&["--foreground", "utun"])
            .env("WG_TUN_NAME_FILE", value)
            .env("LOG_LEVEL", "debug")
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
                    log::error!("[!] wireguard-go failed: {}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                log::error!("[!] wireguard-go failed: {}", e);
            }
        };

        log::info!("[+] wireguard-go exit");
    });

    sleep(Duration::from_millis(500));

    let iface = fs::read_to_string(&name_file)?.trim().to_string();
    Ok(iface)
}

pub fn del_interface(iface: &str, interface: &str) -> TunnelResult<()> {
    let sock_file = format!("/var/run/wireguard/{}.sock", iface);
    let _ = fs::remove_file(sock_file);

    let name_file = format!("/var/run/wireguard/{}.name", interface);
    let _ = fs::remove_file(name_file);

    Ok(())
}

pub fn add_address(iface: &str, addr: &str, is_ipv6: bool) -> TunnelResult<()> {
    if is_ipv6 {
        wg_quick::cmd(&["ifconfig", iface, "inet6", addr, "alias"])?;
    } else {
        let ip = addr.split('/').next().unwrap();
        wg_quick::cmd(&["ifconfig", iface, "inet", addr, ip, "alias"])?;
    }

    Ok(())
}

pub fn set_mtu_and_up(iface: &str, mtu: &EnabledValue) -> TunnelResult<()> {

    let mtu_val = if mtu.enabled {
        mtu.value.to_string()
    } else {
        calculate_mtu().unwrap_or(1420).to_string()
    };

    wg_quick::cmd(&["ifconfig", iface, "mtu", &mtu_val])?;
    wg_quick::cmd(&["ifconfig", iface, "up"])?;

    Ok(())
}

fn calculate_mtu() -> TunnelResult<u16> {
    let output = wg_quick::cmd(&["netstat", "-nr", "-f", "inet"])?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 5 && parts[0] == "default" {
            let default_iface = parts[5];
            let ifconfig_output = wg_quick::cmd(&["ifconfig", default_iface])?;

            let ifconfig_str = String::from_utf8_lossy(&ifconfig_output.stdout);
            for line in ifconfig_str.lines() {
                if line.contains("mtu") {
                    if let Some(mtu_str) = line.split("mtu").nth(1) {
                        if let Some(mtu) = mtu_str.trim().split_whitespace().next() {
                            if let Ok(mtu_val) = mtu.parse::<u16>() {
                                return Ok(mtu_val.saturating_sub(80));
                            }
                        }
                    }
                }
            }
        }
    }

    Err(TunnelError::MtuError())
}

pub fn set_dns(dns_servers: Vec<&str>, _interface: &str) -> TunnelResult<()> {
    let output = wg_quick::cmd(&["networksetup", "-listallnetworkservices"])?;

    let services = String::from_utf8_lossy(&output.stdout);

    for service in services.lines().skip(1) {
        let service = service.trim_start_matches('*').trim();

        let mut set_dns_cmd = vec!["networksetup", "-setdnsservers", service];
        set_dns_cmd.extend(dns_servers.iter().copied());
        let _ = wg_quick::cmd(&set_dns_cmd)?;
    }

    Ok(())
}

pub fn del_dns(_interface: &str) -> TunnelResult<()> {
    let output = wg_quick::cmd(&["networksetup", "-listallnetworkservices"])?;

    let services = String::from_utf8_lossy(&output.stdout);

    for service in services.lines().skip(1) {
        let service = service.trim_start_matches('*').trim();
        let _ = wg_quick::cmd(&["networksetup", "-setdnsservers", service, "Empty"])?;
    }

    Ok(())
}

pub fn add_route(iface: &str, _interface_name: &str, ip: &str, is_default: bool, is_ipv6: bool) -> TunnelResult<()> {

    if is_default {
        if is_ipv6 {
            wg_quick::cmd(&["route", "-q", "-n", "add", "-inet6", "::/1", "-interface", iface])?;
            wg_quick::cmd(&["route", "-q", "-n", "add", "-inet6", "8000::/1", "-interface", iface])?;
        } else {
            wg_quick::cmd(&["route", "-q", "-n", "add", "-inet", "0.0.0.0/1", "-interface", iface])?;
            wg_quick::cmd(&["route", "-q", "-n", "add", "-inet", "128.0.0.0/1", "-interface", iface])?;
        }

       set_endpoint_direct_routes(iface)?;
    } else {
        let family = if is_ipv6 { "-inet6" } else { "-inet" };
        let _ = wg_quick::cmd(&["route", "-q", "-n", "add", family, &ip, "-interface", iface]);
    }

    Ok(())
}

fn set_endpoint_direct_routes(iface: &str) -> TunnelResult<()> {
    let gateway4 = get_default_gateway(false);
    let gateway6 = get_default_gateway(true);
    let endpoints = wg_quick::get_endpoints(iface);

    for endpoint in endpoints {
        if endpoint.contains(':') {
            match &gateway6 {
                Ok(gw) => {
                    wg_quick::cmd(&["route", "-q", "-n", "add", "-inet6", &endpoint, "-gateway", gw])?;
                }
                Err(_) => match &gateway4 {
                    Ok(gw) => {
                        wg_quick::cmd(&["route", "-q", "-n", "add", "-inet", &endpoint, "-gateway", gw])?;
                    }
                    Err(_) => {}
                }
            }
        }
    }
    Ok(())
}
fn get_default_gateway(ipv6: bool) -> TunnelResult<String> {
    let family = if ipv6 { "inet6" } else { "inet" };

    let output = wg_quick::cmd(&["netstat", "-nr", "-f", family])?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 1 && parts[0] == "default" && !parts[1].starts_with("link#") {
            return Ok(parts[1].to_string());
        }
    }
    if !ipv6 {
        Err(TunnelError::IPv4GatewayError())
    } else {
        Err(TunnelError::IPv6GatewayError())
    }
}

pub fn start_monitor_daemon(iface: &str, interface_name: &str, dns: &EnabledValue, mtu: &EnabledValue) -> TunnelResult<()> {

    let has_dns = dns.enabled && !dns.value.is_empty();
    let dns_servers: Vec<String> = if has_dns {
        dns.value.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        Vec::new()
    };
    let has_default_route = wg_quick::get_allowed_ips(iface)?.iter().any(|ip| ip.ends_with("/0"));
    let mtu_auto = !mtu.enabled;

    let iface_clone = iface.to_string();
    let interface_name_clone = interface_name.to_string();
    std::thread::spawn(move || {
        let result = monitor_daemon_worker(
            iface_clone,
            interface_name_clone,
            has_dns,
            dns_servers,
            has_default_route,
            mtu_auto,
        );

        if let Err(e) = result {
            log::warn!("[!] Monitor daemon error: {}", e);
        }
    });

    log::info!("[+] Backgrounding route monitor");
    Ok(())
}

fn monitor_daemon_worker(
    real_iface: String,
    _interface: String,
    has_dns: bool,
    dns_servers: Vec<String>,
    has_default_route: bool,
    mtu_auto: bool,
) -> TunnelResult<()> {
    use std::process::Stdio;
    use std::io::{BufRead, BufReader};

    let mut route_monitor = Command::new("route")
        .args(&["-n", "monitor"])
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = route_monitor.stdout.take()
        .ok_or_else(|| TunnelError::CommandFailed("Failed to capture route monitor output".to_string()))?;

    let reader = BufReader::new(stdout);
    let mut last_dns_update = std::time::Instant::now();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if !line.starts_with("RTM_") {
            continue;
        }

        let iface_check = wg_quick::cmd(&["ifconfig", &real_iface]);

        if iface_check.is_err() || !iface_check?.status.success() {
            log::warn!("[!] Interface {} no longer exists, stopping monitor", &real_iface);
            break;
        }

        if has_default_route {
            if let Err(e) = reapply_endpoint_routes(&real_iface) {
                log::warn!("[!] Failed to reapply endpoint routes: {}", e);
            }
        }

        if mtu_auto {
            if let Err(e) = reapply_mtu(&real_iface) {
                log::warn!("[!] Failed to reapply MTU: {}", e);
            }
        }

        if has_dns && last_dns_update.elapsed() > std::time::Duration::from_secs(2) {
            if let Err(e) = reapply_dns(&dns_servers) {
                log::warn!("[!] Failed to reapply DNS: {}", e);
            }
            last_dns_update = std::time::Instant::now();
        }
    }

    log::info!("[+] Stopping route monitor");

    let _ = route_monitor.kill();
    log::info!("[+] Stopped route monitor");

    Ok(())
}

fn reapply_endpoint_routes(real_iface: &str) -> TunnelResult<()> {
    let gateway4 = get_default_gateway(false);
    let gateway6 = get_default_gateway(true);

    let output = wg_quick::cmd(&["wg", "show", real_iface, "endpoints"])?;

    if !output.status.success() {
        return Ok(());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 1 {
            if let Some(endpoint) = wg_quick::extract_ip_from_endpoint(parts[1]) {
                if endpoint.contains(':') {
                    let _ = wg_quick::cmd(&["route", "-q", "-n", "delete", "-inet6", &endpoint]);

                    if let Ok(gw) = &gateway6 {
                        let _ = wg_quick::cmd(&["route", "-q", "-n", "add", "-inet6", &endpoint, "-gateway", gw]);

                    }
                } else {
                    let _ = wg_quick::cmd(&["route", "-q", "-n", "delete", "-inet", &endpoint]);

                    if let Ok(gw) = &gateway4 {
                        let _ = wg_quick::cmd(&["route", "-q", "-n", "add", "-inet", &endpoint, "-gateway", gw]);
                    }
                }
            }
        }
    }

    Ok(())
}

fn reapply_mtu(real_iface: &str) -> TunnelResult<()> {
    let output = wg_quick::cmd(&["ifconfig", real_iface])?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut current_mtu = None;

    for line in output_str.lines() {
        if line.contains("mtu") {
            if let Some(mtu_str) = line.split("mtu").nth(1) {
                if let Some(mtu) = mtu_str.trim().split_whitespace().next() {
                    current_mtu = mtu.parse::<u16>().ok();
                    break;
                }
            }
        }
    }

    let output = wg_quick::cmd(&["netstat", "-nr", "-f", "inet"])?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 5 && parts[0] == "default" {
            let default_iface = parts[5];
            let ifconfig_output = wg_quick::cmd(&["ifconfig", default_iface])?;

            let ifconfig_str = String::from_utf8_lossy(&ifconfig_output.stdout);
            for line in ifconfig_str.lines() {
                if line.contains("mtu") {
                    if let Some(mtu_str) = line.split("mtu").nth(1) {
                        if let Some(mtu) = mtu_str.trim().split_whitespace().next() {
                            if let Ok(mtu_val) = mtu.parse::<u16>() {
                                let new_mtu = mtu_val.saturating_sub(80);

                                if current_mtu != Some(new_mtu) {
                                    let _ = wg_quick::cmd(&["ifconfig", real_iface, "mtu", &new_mtu.to_string()])?;
                                }
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn reapply_dns(dns_servers: &[String]) -> TunnelResult<()> {
    let output = wg_quick::cmd(&["networksetup", "-listallnetworkservices"])?;

    let services = String::from_utf8_lossy(&output.stdout);

    for service in services.lines().skip(1) {
        let service = service.trim_start_matches('*').trim();

        let mut args = vec!["-setdnsservers", service];
        let dns_refs: Vec<&str> = dns_servers.iter().map(|s| s.as_str()).collect();
        args.extend(dns_refs);

        let mut command = vec!["networksetup"];
        command.extend(args);
        let _ = wg_quick::cmd(&command)?;
    }

    Ok(())
}

pub fn del_routes(iface: &str) -> TunnelResult<()> {
    let output = wg_quick::cmd(&["netstat", "-nr", "-f", "inet"])?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 5 && parts[5] == iface {
            let dest = parts[0];
            let _ = wg_quick::cmd(&["route", "-q", "-n", "delete", "-inet", dest]);
        }
    }

    let output = wg_quick::cmd(&["netstat", "-nr", "-f", "inet6"])?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 3 && parts[3] == iface {
            let dest = parts[0];
            let _ = wg_quick::cmd(&["route", "-q", "-n", "delete", "-inet6", dest]);
        }
    }

    Ok(())
}
