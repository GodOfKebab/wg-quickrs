#![cfg(target_os = "linux")]
use std::io::Write;
use std::process::Command;
use std::fs;
use std::path::Path;
use std::env;
use std::net::Ipv4Addr;
use std::os::unix::prelude::CommandExt;
use std::time::Duration;
use log::{log_enabled, Level};
use regex::Regex;
use wg_quickrs_lib::types::network::Mtu;
use crate::helpers::shell_cmd;
use crate::wireguard::wg_quick;
use crate::wireguard::wg_quick::{DnsManager, TunnelError, TunnelResult};

pub fn interface_exists(interface: &str) -> TunnelResult<Option<String>> {
    let output = shell_cmd(&["ip", "link", "show", "dev", interface]);

    if output.is_ok() {
        return Ok(Some(interface.to_string()));
    }
    Ok(None)
}

pub fn add_interface(interface: &str, userspace: bool, userspace_binary: &str) -> TunnelResult<String> {
    if userspace {
        let interface_cloned = interface.to_string();
        let log_level = if log_enabled!(Level::Debug) { "debug" } else { "error" };
        let userspace_binary_cloned = userspace_binary.to_string();
        std::thread::spawn(move || unsafe {
            match Command::new(&userspace_binary_cloned)
                .args(["--foreground", &interface_cloned])
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
    } else {
        let result = shell_cmd(&["ip", "link", "add", interface, "type", "wireguard"]);

        if result.is_err() {
            log::error!("[!] Missing WireGuard kernel module. Please install and/or load the WireGuard kernel module and try again.");
            return Err(TunnelError::CommandFailed(
                "Failed to create WireGuard interface".to_string()
            ));
        }
    }

    Ok(interface.to_string())
}

pub fn add_address(iface: &str, addr: &str, is_ipv6: bool) -> TunnelResult<()> {
    let proto = if is_ipv6 { "-6" } else { "-4" };
    shell_cmd(&["ip", proto, "address", "add", addr, "dev", iface])?;
    Ok(())
}

pub fn set_mtu_and_up(wg: &str, iface: &str, mtu: &Mtu) -> TunnelResult<()> {
    let mtu_val = if mtu.enabled {
        mtu.value.to_string()
    } else {
        calculate_mtu(wg, iface).unwrap_or(1420).to_string()
    };

    shell_cmd(&["ip", "link", "set", "mtu", &mtu_val, "up", "dev", iface])?;

    Ok(())
}

fn calculate_mtu(wg: &str, iface: &str) -> TunnelResult<u16> {
    let endpoints = wg_quick::get_endpoints(wg, iface);
    let mut min_mtu = u16::MAX;

    // Regex patterns
    let mtu_regex = Regex::new(r"mtu (\d+)").unwrap();
    let dev_regex = Regex::new(r"dev ([^ ]+)").unwrap();

    for endpoint in endpoints {
        let output = shell_cmd(&["ip", "route", "get", &endpoint])?;
        let output_str = String::from_utf8_lossy(&output.stdout);

        if let Some(mtu) = extract_mtu(&output_str, &mtu_regex, &dev_regex) {
            if mtu < min_mtu {
                min_mtu = mtu;
            }
        }
    }

    if min_mtu == u16::MAX {
        if let Ok(default_output) = shell_cmd(&["ip", "route", "show", "default"]) {
            let default_output_str = String::from_utf8_lossy(&default_output.stdout);

            if let Some(mtu) = extract_mtu(&default_output_str, &mtu_regex, &dev_regex) {
                if mtu < min_mtu {
                    min_mtu = mtu;
                }
            }
        }
    }

    if !(min_mtu > 80 && min_mtu < u16::MAX) {
        min_mtu = 1500;
    }

    Ok(min_mtu.saturating_sub(80))
}

fn extract_mtu(output: &str, mtu_regex: &Regex, dev_regex: &Regex) -> Option<u16> {
    // Try to extract MTU directly from output
    if let Some(caps) = mtu_regex.captures(output) {
        if let Ok(mtu) = caps[1].parse::<u16>() {
            return Some(mtu);
        }
    }

    // If not found, try to get device name and query it
    if let Some(caps) = dev_regex.captures(output) {
        let device = &caps[1];
        if let Ok(link_output) = shell_cmd(&["ip", "link", "show", "dev", device]) {
            let link_str = String::from_utf8_lossy(&link_output.stdout);
            if let Some(caps) = mtu_regex.captures(&link_str) {
                if let Ok(mtu) = caps[1].parse::<u16>() {
                    return Some(mtu);
                }
            }
        }
    }

    None
}

fn resolvconf_iface_prefix() -> String {
    let interface_order_path = Path::new("/etc/resolvconf/interface-order");
    if !interface_order_path.is_file() {
        return String::new();
    }

    // Find resolvconf in PATH
    let mut found_non_symlink = false;
    if let Ok(path_var) = env::var("PATH") {
        for dir in path_var.split(':') {
            let resolvconf_path = Path::new(dir).join("resolvconf");
            if let Ok(metadata) = fs::symlink_metadata(&resolvconf_path) {
                // If it's a symlink, return empty
                if metadata.file_type().is_symlink() {
                    return String::new();
                }
                found_non_symlink = true;
                break;
            }
        }
    }

    if !found_non_symlink {
        return String::new();
    }

    // Read the interface-order file and find a prefix
    // Matches pattern like "tun*" and captures "tun"
    let re = Regex::new(r"^([A-Za-z0-9-]+)\*$").unwrap();

    if let Ok(contents) = fs::read_to_string(interface_order_path) {
        for line in contents.lines() {
            let line = line.trim();
            if let Some(caps) = re.captures(line) {
                // Return the captured prefix with a dot
                return format!("{}.", &caps[1]);
            }
        }
    }

    String::new()
}

pub fn set_dns(dns_servers: &Vec<Ipv4Addr>, interface: &str, dns_manager: &mut DnsManager) -> TunnelResult<()> {
    dns_manager.have_set_dns = false;
    if dns_servers.is_empty() {
        return Ok(());
    }

    let dns_config = dns_servers
        .iter()
        .map(|s| format!("nameserver {}\n", s))
        .collect::<String>();

    log::debug!("[+] resolvconf -a {}{} -m 0 -x", resolvconf_iface_prefix(), interface);
    let mut child = Command::new("resolvconf")
        .args(&["-a", &format!("{}{}", resolvconf_iface_prefix(), interface), "-m", "0", "-x"])
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(dns_config.as_bytes())?;
    }

    child.wait()?;

    dns_manager.have_set_dns = true;
    Ok(())
}

pub fn add_route(wg: &str, iface: &str, interface_name: &str, cidr: &str, endpoint_router: &mut wg_quick::EndpointRouter) -> TunnelResult<()> {
    endpoint_router.have_set_firewall = false;
    let is_default = cidr.ends_with("/0");
    
    if is_default {
        add_default_route(wg, interface_name, cidr)?;
        endpoint_router.have_set_firewall = true;
    } else {
        let is_ipv6 = cidr.contains(':');
        let proto = if is_ipv6 { "-6" } else { "-4" };

        let check = shell_cmd(&["ip", proto, "route", "show", "dev", iface, "match", &cidr])?;
        if check.stdout.is_empty() {
            shell_cmd(&["ip", proto, "route", "add", &cidr, "dev", iface])?;
        }
    }

    Ok(())
}

fn get_fwmark(wg: &str, interface: &str) -> TunnelResult<u16> {
    let output = shell_cmd(&[wg, "show", interface, "fwmark"])?;
    let fwmark_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if fwmark_str.is_empty() || fwmark_str == "off" {
        return Err(TunnelError::InvalidConfig("No fwmark set".into()));
    }

    fwmark_str.parse().map_err(|e| {
        TunnelError::InvalidConfig(format!("Invalid fwmark value: {}", e))
    })
}

fn find_unused_table() -> TunnelResult<u16> {
    let mut table = 51820u16;

    loop {
        let ipv4_check = shell_cmd(&["ip", "-4", "route", "show", "table", &table.to_string()])?;
        let ipv6_check = shell_cmd(&["ip", "-6", "route", "show", "table", &table.to_string()])?;

        if ipv4_check.stdout.is_empty() && ipv6_check.stdout.is_empty() {
            return Ok(table);
        }

        table += 1;

        if table == u16::MAX {
            return Err(TunnelError::InvalidConfig(
                "No available routing table found".into()
            ));
        }
    }
}

pub fn add_default_route(wg: &str, interface: &str, cidr: &str) -> TunnelResult<()> {
    // Get or create fwmark/table
    let table = &match get_fwmark(wg, interface) {
        Ok(mark) => mark,
        Err(_) => {
            let table = find_unused_table()?;
            shell_cmd(&[wg, "set", interface, "fwmark", &table.to_string()])?;
            table
        }
    }.to_string();

    // Detect IPv4 vs IPv6
    let is_ipv6 = cidr.contains(':');
    let proto = if is_ipv6 { "-6" } else { "-4" };
    let iptables = if is_ipv6 { "ip6tables" } else { "iptables" };
    let pf = if is_ipv6 { "ip6" } else { "ip" };

    // Add routing rules
    shell_cmd(&["ip", proto, "rule", "add", "not", "fwmark", table, "table", table])?;
    shell_cmd(&["ip", proto, "rule", "add", "table", "main", "suppress_prefixlength", "0"])?;
    shell_cmd(&["ip", proto, "route", "add", cidr, "dev", interface, "table", table])?;

    // Build firewall rules
    let marker = format!("-m comment --comment \"wg-quickrs rule for {}\"", interface);
    let mut restore = String::from("*raw\n");

    let nftable = format!("wg-quickrs-{}", interface);
    let mut nftcmd = String::new();

    // Setup nftables structure
    nftcmd.push_str(&format!("add table {} {}\n", pf, nftable));
    nftcmd.push_str(&format!(
        "add chain {} {} preraw {{ type filter hook prerouting priority -300; }}\n",
        pf, nftable
    ));
    nftcmd.push_str(&format!(
        "add chain {} {} premangle {{ type filter hook prerouting priority -150; }}\n",
        pf, nftable
    ));
    nftcmd.push_str(&format!(
        "add chain {} {} postmangle {{ type filter hook postrouting priority -150; }}\n",
        pf, nftable
    ));

    // Get interface addresses and add anti-spoofing rules
    let addr_output = shell_cmd(&["ip", "-o", proto, "addr", "show", "dev", interface])?;

    let addr_str = String::from_utf8_lossy(&addr_output.stdout);
    let addr_regex = Regex::new(r".*inet6?\\ ([0-9a-f:.]+)/[0-9]+.*")
        .map_err(|e| TunnelError::InvalidConfig(format!("Regex error: {}", e)))?;

    for line in addr_str.lines() {
        if let Some(caps) = addr_regex.captures(line) {
            let ip_addr = &caps[1];

            // iptables rule
            restore.push_str(&format!(
                "-I PREROUTING ! -i {} -d {} -m addrtype ! --src-type LOCAL -j DROP {}\n",
                interface, ip_addr, marker
            ));

            // nftables rule
            nftcmd.push_str(&format!(
                "add rule {} {} preraw iifname != \"{}\" {} daddr {} fib saddr type != local drop\n",
                pf, nftable, interface, pf, ip_addr
            ));
        }
    }

    // Connection marking rules
    restore.push_str(&format!(
        "COMMIT\n*mangle\n-I POSTROUTING -m mark --mark {} -p udp -j CONNMARK --save-mark {}\n",
        table, marker
    ));
    restore.push_str(&format!(
        "-I PREROUTING -p udp -j CONNMARK --restore-mark {}\nCOMMIT\n",
        marker
    ));

    nftcmd.push_str(&format!(
        "add rule {} {} postmangle meta l4proto udp mark {} ct mark set mark\n",
        pf, nftable, table
    ));
    nftcmd.push_str(&format!(
        "add rule {} {} premangle meta l4proto udp meta mark set ct mark\n",
        pf, nftable
    ));

    // Enable source validation for IPv4
    if !is_ipv6 {
        let _ = shell_cmd(&["sysctl", "-q", "net.ipv4.conf.all.src_valid_mark=1"]);
    }

    // Apply firewall rules - prefer nftables if available
    let nft_exists = shell_cmd(&["nft", "--version"])
        .map(|o| o.status.success())
        .unwrap_or(false);

    if nft_exists {
        execute_nft_command(&nftcmd)?;
    } else {
        execute_iptables_command(&iptables, &restore)?;
    }

    Ok(())
}

fn execute_nft_command(nftcmd: &str) -> TunnelResult<()> {
    let mut nft = Command::new("nft")
        .args(["-f", "-"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| TunnelError::CommandFailed(format!("Failed to spawn nft: {}", e)))?;

    if let Some(mut stdin) = nft.stdin.take() {
        stdin.write_all(nftcmd.as_bytes())?;
    }

    let status = nft.wait()?;
    if !status.success() {
        return Err(TunnelError::CommandFailed(
            "nft command failed".into()
        ));
    }

    Ok(())
}

fn execute_iptables_command(iptables: &str, restore: &str) -> TunnelResult<()> {
    let mut iptables_restore = Command::new(format!("{}-restore", iptables))
        .arg("-n")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| TunnelError::CommandFailed(format!("Failed to spawn iptables-restore: {}", e)))?;

    if let Some(mut stdin) = iptables_restore.stdin.take() {
        stdin.write_all(restore.as_bytes())?;
    }

    let status = iptables_restore.wait()?;
    if !status.success() {
        return Err(TunnelError::CommandFailed(
            "iptables-restore command failed".into()
        ));
    }

    Ok(())
}

fn remove_nftables(interface: &str) -> TunnelResult<()> {
    // Check if nft is available
    let nft_exists = shell_cmd(&["nft", "--version"])
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !nft_exists {
        return Ok(());
    }

    // List all nftables tables
    let output = shell_cmd(&["nft", "list", "tables"])?;

    let tables_str = String::from_utf8_lossy(&output.stdout);
    let target_table = format!("wg-quickrs-{}", interface);
    let mut nftcmd = String::new();

    // Find and delete matching tables
    for line in tables_str.lines() {
        if line.contains(&target_table) {
            nftcmd.push_str(&format!("delete {}\n", line.trim()));
        }
    }

    // Execute delete commands if any were found
    if !nftcmd.is_empty() {
        execute_nft_command(&nftcmd)?;
    }

    Ok(())
}

fn remove_iptables(interface: &str) -> TunnelResult<()> {
    // Check if iptables is available
    let iptables_exists = shell_cmd(&["iptables", "--version"])
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !iptables_exists {
        return Ok(());
    }

    let marker = format!("-m comment --comment \"wg-quickrs rule for {}\"", interface);

    // Process both iptables and ip6tables
    for iptables in &["iptables", "ip6tables"] {
        // Get current rules
        let save_output = shell_cmd(&[&format!("{}-save", iptables)])?;

        let rules_str = String::from_utf8_lossy(&save_output.stdout);
        let mut restore = String::new();
        let mut found = false;

        // Build delete commands
        for line in rules_str.lines() {
            // Keep table declarations, COMMIT statements, and our marked rules
            if line.starts_with('*') || line == "COMMIT" ||
                (line.starts_with("-A ") && line.contains(&marker)) {

                if line.starts_with("-A ") {
                    found = true;
                    // Convert -A (append) to -D (delete)
                    restore.push_str(&line.replacen("-A ", "-D ", 1));
                } else {
                    restore.push_str(line);
                }
                restore.push('\n');
            }
        }

        // Apply deletions if any rules were found
        if found {
            execute_iptables_command(iptables, &restore)?;
        }
    }

    Ok(())
}

fn remove_routing_rules(table: u16) -> TunnelResult<()> {
    for proto in ["-4", "-6"] {
        // Remove IPv4/IPv6 rules
        remove_rules_matching(proto, &format!("lookup {}", table), &[
            "ip", proto, "rule", "delete", "table", &table.to_string()
        ])?;

        // Remove IPv4/IPv6 suppress rules
        remove_rules_matching(proto, "from all lookup main suppress_prefixlength 0", &[
            "ip", proto, "rule", "delete", "table", "main", "suppress_prefixlength", "0"
        ])?;
    }

    Ok(())
}

fn remove_rules_matching(proto: &str, pattern: &str, delete_cmd: &[&str]) -> TunnelResult<()> {
    loop {
        let check = shell_cmd(&["ip", proto, "rule", "show"])?;
        let rules_str = String::from_utf8_lossy(&check.stdout);
        if !rules_str.contains(pattern) {
            break;
        }

        shell_cmd(delete_cmd)?;
    }

    Ok(())
}

pub fn del_interface(wg: &str, _iface: &str, interface: &str, dns_manager: &mut DnsManager, endpoint_router: &mut wg_quick::EndpointRouter) -> TunnelResult<()> {
    // Unset DNS if it was configured
    if dns_manager.have_set_dns {
        del_dns(interface, dns_manager)?;
    }

    // Remove firewall and routing rules if they were configured
    if endpoint_router.have_set_firewall {
        remove_nftables(interface)?;
        remove_iptables(interface)?;
        if let Ok(table) = get_fwmark(wg, interface) {
            remove_routing_rules(table)?;
        } else {
            log::warn!("Failed to get fwmark for interface {}. Skipping routing rules cleanup.", interface);
        }
        endpoint_router.have_set_firewall = false;
    }

    // Delete the interface
    shell_cmd(&["ip", "link", "delete", "dev", interface])?;
    Ok(())
}

pub fn del_routes(_iface: &str) -> TunnelResult<()> {
    // Routes are automatically cleaned up when the interface is deleted
    Ok(())
}

pub fn del_dns(interface: &str, dns_manager: &mut DnsManager) -> TunnelResult<()> {
    if !dns_manager.have_set_dns {
        return Ok(());
    }
    let _ = shell_cmd(&["resolvconf", "-d", &format!("{}{}", resolvconf_iface_prefix(), interface), "-f"])?;
    dns_manager.have_set_dns = false;

    Ok(())
}
