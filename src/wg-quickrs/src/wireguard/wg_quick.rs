use std::process::{Command, Output};
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;
use thiserror::Error;

use wg_quickrs_wasm::types::{Config, EnabledValue};
use crate::macros::full_version;
#[cfg(target_os = "macos")]
use crate::wireguard::wg_quick_darwin as wg_quick_platform;
#[cfg(target_os = "linux")]
use crate::wireguard::wg_quick_linux as wg_quick_platform;


#[derive(Error, Debug)]
pub enum TunnelError {
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Interface already exists: {0}")]
    InterfaceExists(String),
    #[error("Interface not found: {0}")]
    InterfaceNotFound(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Key error: {0}")]
    KeyError(#[from] wg_quickrs_wasm::types::WireGuardLibError),
    #[cfg(target_os = "macos")]
    #[error("Unable to parse MTU")]
    MtuError(),
    #[cfg(target_os = "macos")]
    #[error("Unable to parse IPv4 gateway")]
    IPv4GatewayError(),
    #[cfg(target_os = "macos")]
    #[error("Unable to parse IPv6 gateway")]
    IPv6GatewayError(),
}

pub type TunnelResult<T> = Result<T, TunnelError>;

pub struct TunnelManager {
    config: Config,
    real_interface: Option<String>,
}

impl TunnelManager {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            real_interface: None,
        }
    }

    fn interface_name(&self) -> &str {
        &self.config.network.identifier
    }

    pub fn start_tunnel(&mut self) -> TunnelResult<String> {
        if self.interface_exists()? {
            return Err(TunnelError::InterfaceExists(self.interface_name().to_string()));
        }

        self.execute_hooks(HookType::PreUp)?;
        self.add_interface()?;
        self.set_config()?;
        self.add_addresses()?;
        self.set_mtu_and_up()?;
        self.add_routes()?;
        self.set_dns()?;

        #[cfg(target_os = "macos")]
        {
            let iface = self.real_interface.as_ref().unwrap();
            let interface = self.interface_name();
            let this_peer = self.config.network.peers.get(&self.config.network.this_peer)
                .ok_or_else(|| TunnelError::InvalidConfig("This peer not found".to_string()))?;

            wg_quick_platform::start_monitor_daemon(iface, interface, &this_peer.dns, &this_peer.mtu)?;
        }

        self.execute_hooks(HookType::PostUp)?;

        Ok(self.real_interface.clone().unwrap_or_else(|| self.interface_name().to_string()))
    }

    pub fn stop_tunnel(&mut self) -> TunnelResult<()> {
        if !self.interface_exists()? {
            log::warn!("Interface already deleted, skipping cleanup");
            return Ok(());
        }

        if !self.is_wireguard_interface()? {
            return Err(TunnelError::InvalidConfig(format!(
                "'{}' is not a WireGuard interface",
                self.interface_name()
            )));
        }

        let _ = self.execute_hooks(HookType::PreDown);
        let _ = self.del_routes();
        let _ = self.del_dns();
        let _ = self.del_interface();
        let _ = self.execute_hooks(HookType::PostDown);
        Ok(())
    }

    fn interface_exists(&mut self) -> TunnelResult<bool> {
        let interface = self.interface_name();

        #[cfg(target_os = "macos")]
        {
            let name_file = format!("/var/run/wireguard/{}.name", interface);
            if !std::path::PathBuf::from(&name_file).exists() {
                return Ok(false);
            }

            let iface = fs::read_to_string(&name_file)?.trim().to_string();
            let sock_file = format!("/var/run/wireguard/{}.sock", iface);

            if std::path::PathBuf::from(&sock_file).exists() {
                self.real_interface = Some(iface);
                return Ok(true);
            }
            Ok(false)
        }

        #[cfg(target_os = "linux")]
        {
            let output = cmd(&["ip", "link", "show", "dev", interface])?;

            if output.status.success() {
                self.real_interface = Some(interface.to_string());
            }
            Ok(output.status.success())
        }
    }

    fn add_interface(&mut self) -> TunnelResult<()> {
        let interface = self.interface_name();
        self.real_interface = Some(wg_quick_platform::add_interface(interface)?);
        Ok(())
    }

    fn del_interface(&mut self) -> TunnelResult<()> {
        let iface = self.real_interface.as_ref().ok_or_else(|| {
            TunnelError::InterfaceNotFound("No interface to delete".to_string())
        })?;
        let interface = self.interface_name();
        wg_quick_platform::del_interface(iface, interface)?;

        Ok(())
    }

    fn add_addresses(&self) -> TunnelResult<()> {
        let iface = self.real_interface.as_ref().unwrap();

        let this_peer = self.config.network.peers.get(&self.config.network.this_peer)
            .ok_or_else(|| TunnelError::InvalidConfig("This peer not found".to_string()))?;

        let addresses: Vec<&str> = this_peer.address.split(',').map(|s| s.trim()).collect();

        for addr in addresses {
            let is_ipv6 = addr.contains(':');
            wg_quick_platform::add_address(iface, addr, is_ipv6)?;
        }
        Ok(())
    }

    fn set_mtu_and_up(&self) -> TunnelResult<()> {
        let iface = self.real_interface.as_ref().unwrap();

        let this_peer = self.config.network.peers.get(&self.config.network.this_peer)
            .ok_or_else(|| TunnelError::InvalidConfig("This peer not found".to_string()))?;

        wg_quick_platform::set_mtu_and_up(iface, &this_peer.mtu)?;

        Ok(())
    }

    fn set_dns(&self) -> TunnelResult<()> {
        let this_peer = self.config.network.peers.get(&self.config.network.this_peer)
            .ok_or_else(|| TunnelError::InvalidConfig("This peer not found".to_string()))?;

        if !this_peer.dns.enabled || this_peer.dns.value.is_empty() {
            return Ok(());
        }

        let dns_servers: Vec<&str> = this_peer.dns.value.split(',').map(|s| s.trim()).collect();

        wg_quick_platform::set_dns(dns_servers, self.interface_name())
    }

    fn del_dns(&self) -> TunnelResult<()> {
        wg_quick_platform::del_dns(self.interface_name())
    }

    fn add_routes(&self) -> TunnelResult<()> {
        let iface = self.real_interface.as_ref().unwrap();
        let allowed_ips = get_allowed_ips(iface)?;

        for ip in allowed_ips {
            let is_default = ip.ends_with("/0");
            let is_ipv6 = ip.contains(':');

            wg_quick_platform::add_route(iface, &self.config.network.identifier, &ip, is_default, is_ipv6)?;
        }

        Ok(())
    }

    fn del_routes(&self) -> TunnelResult<()> {
        let iface = self.real_interface.as_ref().ok_or_else(|| {
            TunnelError::InterfaceNotFound("No interface for route deletion".to_string())
        })?;

        wg_quick_platform::del_routes(iface)
    }

    fn set_config(&self) -> TunnelResult<()> {
        let iface = self.real_interface.as_ref().unwrap();
        let wg_config = wg_quickrs_wasm::helpers::get_peer_wg_config(&self.config.network, &self.config.network.this_peer, full_version!(), true, None)?;

        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "{}", wg_config)?;
        cmd(&["wg", "setconf", iface, &temp_file.path().to_string_lossy()])?;
        let _ = fs::remove_file(&temp_file);

        Ok(())
    }

    fn is_wireguard_interface(&self) -> TunnelResult<bool> {
        let output = cmd(&["wg", "show", "interfaces"])?;

        if !output.status.success() {
            return Ok(false);
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let interfaces: Vec<&str> = output_str.split_whitespace().collect();

        let interface_name = self.interface_name();
        let real_iface = self.real_interface.as_ref().map(|s| s.as_str());

        Ok(interfaces.contains(&interface_name) ||
            (real_iface.is_some() && interfaces.contains(&real_iface.unwrap())))
    }

    fn execute_hooks(&self, hook_type: HookType) -> TunnelResult<()> {
        let this_peer = self.config.network.peers.get(&self.config.network.this_peer)
            .ok_or_else(|| TunnelError::InvalidConfig("This peer not found".to_string()))?;

        let fw_utility = &self.config.agent.firewall.utility.to_string_lossy();
        let subnet = &self.config.network.subnet;
        let gateway = &self.config.agent.firewall.gateway;
        let port = &self.config.agent.vpn.port;
        let interface = &self.config.network.identifier;

        let mut cmds: Vec<EnabledValue> = Vec::new();
        let hooks = match hook_type {
            HookType::PreUp => &this_peer.scripts.pre_up,
            HookType::PostUp => {
                if self.config.agent.firewall.enabled && let Some(utility) = self.config.agent.firewall.utility.file_name() {
                    if utility == "iptables" {
                        cmds.push(EnabledValue{
                            enabled: true,
                            value: format!("{fw_utility} -t nat -I POSTROUTING -s {subnet} -o {gateway} -j MASQUERADE;")
                        });
                        cmds.push(EnabledValue{
                            enabled: true,
                            value: format!("{fw_utility} -I INPUT -p udp -m udp --dport {port} -j ACCEPT;")
                        });
                        cmds.push(EnabledValue{
                            enabled: true,
                            value: format!("{fw_utility} -I FORWARD -i {interface} -j ACCEPT;")
                        });
                        cmds.push(EnabledValue{
                            enabled: true,
                            value: format!("{fw_utility} -I FORWARD -o {interface} -j ACCEPT;")
                        });
                        cmds.push(EnabledValue{
                            enabled: true,
                            value: "sysctl -w net.ipv4.ip_forward=1;".to_string()
                        });
                    } else if utility == "pfctl" {
                        let nat_rule = format!("nat on {gateway} from {subnet} to any -> {gateway}",
                                               gateway = self.config.agent.firewall.gateway,
                                               subnet = self.config.network.subnet);
                        cmds.push(EnabledValue{
                            enabled: true,
                            value: format!("awk \"/^nat/ {{print; print \\\"{nat_rule}\\\"; next}}1\" /etc/pf.conf > /etc/pf.conf.new && mv /etc/pf.conf /etc/pf.conf.bak && mv /etc/pf.conf.new /etc/pf.conf;")
                        });
                        cmds.push(EnabledValue{
                            enabled: true,
                            value: format!("grep -qxF '{nat_rule}' /etc/pf.conf || echo '*** could NOT configure firewall because there are no existing NAT rules. See notes at docs/MACOS-FIREWALL.md ' >&2;")
                        });
                        cmds.push(EnabledValue{
                            enabled: true,
                            value: format!("grep -qxF '{nat_rule}' /etc/pf.conf || exit 1;")
                        });
                        cmds.push(EnabledValue{
                            enabled: true,
                            value: format!("{fw_utility} -f /etc/pf.conf;")
                        });
                        cmds.push(EnabledValue{
                            enabled: true,
                            value: format!("{fw_utility} -e || true;")
                        });
                        cmds.push(EnabledValue{
                            enabled: true,
                            value: "sysctl -w net.inet.ip.forwarding=1;".to_string()
                        });
                    }
                }
                cmds.extend(this_peer.scripts.post_up.clone());
                &cmds
            },
            HookType::PreDown => &this_peer.scripts.pre_down,
            HookType::PostDown => {
                if self.config.agent.firewall.enabled && let Some(utility) = self.config.agent.firewall.utility.file_name() {
                    if utility == "iptables" {
                        cmds.push(EnabledValue{
                            enabled: true,
                            value: format!("{fw_utility} -t nat -D POSTROUTING -s {subnet} -o {gateway} -j MASQUERADE;")
                        });
                        cmds.push(EnabledValue{
                            enabled: true,
                            value: format!("{fw_utility} -D INPUT -p udp -m udp --dport {port} -j ACCEPT;")
                        });
                        cmds.push(EnabledValue{
                            enabled: true,
                            value: format!("{fw_utility} -D FORWARD -i {interface} -j ACCEPT;")
                        });
                        cmds.push(EnabledValue{
                            enabled: true,
                            value: format!("{fw_utility} -D FORWARD -o {interface} -j ACCEPT;")
                        });
                        #[cfg(not(feature = "docker"))]
                        cmds.push(EnabledValue{
                            enabled: true,
                            value: "sysctl -w net.ipv4.ip_forward=0;".to_string()
                        });
                    } else if utility == "pfctl" {
                        let nat_rule = format!("nat on {gateway} from {subnet} to any -> {gateway}",
                                               gateway = self.config.agent.firewall.gateway,
                                               subnet = self.config.network.subnet);
                        cmds.push(EnabledValue{
                            enabled: true,
                            value: format!("awk -v line='{nat_rule}' '$0 != line' /etc/pf.conf > /etc/pf.conf.new && mv /etc/pf.conf /etc/pf.conf.bak && mv /etc/pf.conf.new /etc/pf.conf;")
                        });
                        cmds.push(EnabledValue{
                            enabled: true,
                            value: format!("{fw_utility} -d || true;")
                        });
                        cmds.push(EnabledValue{
                            enabled: true,
                            value: "sysctl -w net.ipv4.ip_forwarding=0;".to_string()
                        });
                    }
                }
                cmds.extend(this_peer.scripts.post_down.clone());
                &cmds
            },
        };

        for hook in hooks {
            if !hook.enabled {
                continue;
            }

            log::info!("[+] {}", hook.value);
            let output = cmd(&["sh", "-c", &hook.value])?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log::warn!("Warning: Hook failed: {}", stderr);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum HookType {
    PreUp,
    PostUp,
    PreDown,
    PostDown,
}

pub fn extract_ip_from_endpoint(endpoint: &str) -> Option<String> {
    if endpoint.starts_with('[') {
        if let Some(end) = endpoint.find(']') {
            return Some(endpoint[1..end].to_string());
        }
    }

    if let Some(colon_pos) = endpoint.rfind(':') {
        return Some(endpoint[..colon_pos].to_string());
    }

    None
}

pub fn get_allowed_ips(iface: &str) -> TunnelResult<Vec<String>> {
    let output = cmd(&["wg", "show", iface, "allowed-ips"]);

    if !output.is_ok() {
        log::warn!("Failed to get allowed IPs, assuming no routes");
        return Ok(Vec::new());
    }

    let output = output?;
    let output_str = String::from_utf8_lossy(&*output.stdout);
    let mut ips = Vec::new();

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 1 {
            for ip in &parts[1..] {
                if ip.contains('/') {
                    ips.push(ip.to_string());
                }
            }
        }
    }

    ips.sort_by(|a, b| {
        let prefix_a = a.split('/').nth(1).and_then(|p| p.parse::<u8>().ok()).unwrap_or(0);
        let prefix_b = b.split('/').nth(1).and_then(|p| p.parse::<u8>().ok()).unwrap_or(0);
        prefix_b.cmp(&prefix_a)
    });

    Ok(ips)
}

pub fn get_endpoints(iface: &str) -> Vec<String> {
    let output = match cmd(&["wg", "show", iface, "endpoints"]) {
        Ok(output) => output,
        Err(e) => {
            log::warn!("Failed to get endpoints: {}, defaulting to an empty list of endpoints", e);
            return Vec::new();
        }
    };

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut endpoints = Vec::new();

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 1 {
            if let Some(ip) = extract_ip_from_endpoint(parts[1]) {
                endpoints.push(ip);
            }
        }
    }

    endpoints
}

/// Start a WireGuard tunnel with the given configuration.
/// Returns the real interface name (e.g., "utun3" on macOS or the configured name on Linux).
pub fn start_tunnel(config: Config) -> TunnelResult<String> {
    let mut manager = TunnelManager::new(config);
    manager.start_tunnel()
}

/// Stop a WireGuard tunnel with the given configuration.
pub fn stop_tunnel(config: Config) -> TunnelResult<()> {
    let mut manager = TunnelManager::new(config);
    manager.stop_tunnel()
}

pub fn cmd(args: &[&str]) -> TunnelResult<Output> {
    if args.is_empty() {
        return Err(TunnelError::CommandFailed("Empty command".to_string()));
    }

    log::info!("[+] {}", args.join(" "));

    let output = Command::new(args[0])
        .args(&args[1..])
        .output()?;
    if !output.stderr.is_empty() {
        log::warn!("[!] {}", String::from_utf8_lossy(&output.stderr));
    }

    if !output.status.success() {
        log::info!("[!] {}", String::from_utf8_lossy(&output.stdout));
        return Err(TunnelError::CommandFailed(String::from_utf8_lossy(&output.stderr).to_string()));
    }

    Ok(output)
}
