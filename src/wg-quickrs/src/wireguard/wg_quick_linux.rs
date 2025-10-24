// #![cfg(target_os = "linux")]
use std::io::Write;
use std::process::Command;
use wg_quickrs_wasm::types::EnabledValue;
use crate::wireguard::wg_quick;
use crate::wireguard::wg_quick::{DnsManager, TunnelError, TunnelResult};

pub fn interface_exists(interface: &str) -> TunnelResult<Option<String>> {
    let output = wg_quick::cmd(&["ip", "link", "show", "dev", interface])?;

    if output.status.success() {
        return Ok(Some(interface.to_string()));
    }
    Ok(None)
}

pub fn add_interface(interface: &str) -> TunnelResult<String> {
    let result = wg_quick::cmd(&["ip", "link", "add", "dev", interface, "type", "wireguard"])?;

    if !result.status.success() {
        log::info!("Failed to create WireGuard interface, trying userspace");
        let userspace_result = wg_quick::cmd(&["wireguard-go", interface])?;

        if !userspace_result.status.success() {
            return Err(TunnelError::CommandFailed(
                "Failed to create WireGuard interface".to_string()
            ));
        }
    }

    Ok(interface.to_string())
}

pub fn del_interface(iface: &str, _interface: &str) -> TunnelResult<()> {
    wg_quick::cmd(&["ip", "link", "delete", "dev", iface])?;
    Ok(())
}

pub fn add_address(iface: &str, addr: &str, is_ipv6: bool) -> TunnelResult<()> {
    let proto = if is_ipv6 { "-6" } else { "-4" };
    wg_quick::cmd(&["ip", proto, "address", "add", addr, "dev", iface])?;
    Ok(())
}

pub fn set_mtu_and_up(iface: &str, mtu: &EnabledValue) -> TunnelResult<()> {
    let mtu_val = if mtu.enabled {
        mtu.value.to_string()
    } else {
        calculate_mtu(iface).unwrap_or(1420).to_string()
    };

    wg_quick::cmd(&["ip", "link", "set", "mtu", &mtu_val, "up", "dev", iface])?;

    Ok(())
}

fn calculate_mtu(iface: &str) -> TunnelResult<u16> {
    let endpoints = wg_quick::get_endpoints(iface);
    let mut min_mtu = 2147483647u32;

    for endpoint in endpoints {
        let output = wg_quick::cmd(&["ip", "route", "get", &endpoint])?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("mtu") {
                if let Some(mtu_str) = line.split("mtu").nth(1) {
                    if let Some(mtu) = mtu_str.trim().split_whitespace().next() {
                        if let Ok(mtu_val) = mtu.parse::<u32>() {
                            if mtu_val < min_mtu {
                                min_mtu = mtu_val;
                            }
                        }
                    }
                }
            }
        }
    }

    if min_mtu == 2147483647 {
        min_mtu = 1500;
    }

    Ok((min_mtu.saturating_sub(80)) as u16)
}

pub fn set_dns(dns_servers: &Vec<String>, interface: &str, _dns_manager: &mut DnsManager) -> TunnelResult<()> {
    let mut dns_config = String::new();
    for server in dns_servers {
        dns_config.push_str(&format!("nameserver {}\n", server));
    }

    let mut child = Command::new("resolvconf")
        .args(&["-a", interface, "-m", "0", "-x"])
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(dns_config.as_bytes())?;
    }

    child.wait()?;

    Ok(())
}

pub fn del_dns(interface: &str, _dns_manager: &mut DnsManager) -> TunnelResult<()> {
    let _ = wg_quick::cmd(&["resolvconf", "-d", interface, "-f"])?;

    Ok(())
}

pub fn add_route(iface: &str, interface_name: &str, cidr: &str, _endpoint_router: &mut wg_quick::EndpointRouter) -> TunnelResult<()> {
    let is_default = cidr.ends_with("/0");
    let is_ipv6 = cidr.contains(':');
    let proto = if is_ipv6 { "-6" } else { "-4" };

    if is_default {
        add_default_route(iface, interface_name, &cidr, proto)?;
    } else {
        wg_quick::cmd(&["ip", proto, "route", "add", &cidr, "dev", iface])?;
    }

    Ok(())
}

pub fn set_endpoint_direct_route(_iface: &str, _endpoint_router: &mut wg_quick::EndpointRouter) -> TunnelResult<()> {
    Ok(())
}

fn add_default_route(iface: &str, interface_name: &str, route: &str, proto: &str) -> TunnelResult<()> {
    let table = get_or_create_fwmark_table(iface)?;

    wg_quick::cmd(&["ip", proto, "rule", "add", "not", "fwmark", &table.to_string(), "table", &table.to_string()])?;
    wg_quick::cmd(&["ip", proto, "rule", "add", "table", "main", "suppress_prefixlength", "0"])?;
    wg_quick::cmd(&["ip", proto, "route", "add", route, "dev", iface, "table", &table.to_string()])?;

    setup_firewall(interface_name, table, proto)?;

    Ok(())
}

fn get_or_create_fwmark_table(iface: &str) -> TunnelResult<u32> {
    let output = wg_quick::cmd(&["wg", "show", iface, "fwmark"])?;

    let fwmark_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if fwmark_str.is_empty() || fwmark_str == "off" {
        let mut table = 51820u32;
        loop {
            let v4_check = wg_quick::cmd(&["ip", "-4", "route", "show", "table", &table.to_string()])?;
            let v6_check = wg_quick::cmd(&["ip", "-6", "route", "show", "table", &table.to_string()])?;

            if v4_check.stdout.is_empty() && v6_check.stdout.is_empty() {
                break;
            }
            table += 1;
        }

        wg_quick::cmd(&["wg", "set", iface, "fwmark", &table.to_string()])?;
        Ok(table)
    } else {
        fwmark_str.parse::<u32>()
            .map_err(|_| TunnelError::InvalidConfig("Invalid fwmark".to_string()))
    }
}

fn setup_firewall(interface_name: &str, table: u32, proto: &str) -> TunnelResult<()> {
    let pf = if proto == "-6" { "ip6" } else { "ip" };

    if wg_quick::cmd(&["nft", "--version"]).is_ok() {
        let nftable = format!("wg-quick-{}", interface_name);
        let mut nft_commands = String::new();

        nft_commands.push_str(&format!("add table {} {}\n", pf, nftable));
        nft_commands.push_str(&format!("add chain {} {} preraw {{ type filter hook prerouting priority -300; }}\n", pf, nftable));
        nft_commands.push_str(&format!("add chain {} {} postmangle {{ type filter hook postrouting priority -150; }}\n", pf, nftable));
        nft_commands.push_str(&format!("add rule {} {} postmangle meta l4proto udp mark {} ct mark set mark\n", pf, nftable, table));

        let mut child = Command::new("nft")
            .arg("-f")
            .arg("-")
            .stdin(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(nft_commands.as_bytes())?;
        }
        child.wait()?;
    }

    Ok(())
}

pub fn del_routes(_iface: &str) -> TunnelResult<()> {
    // Routes are automatically cleaned up when interface is deleted
    Ok(())
}
