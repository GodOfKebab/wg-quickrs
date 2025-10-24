use std::collections::HashMap;
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
    #[error("Unable to find default gateway")]
    DefaultGatewayNotFound(),
}

pub type TunnelResult<T> = Result<T, TunnelError>;

#[allow(dead_code)]
pub struct EndpointRouter {
    pub(crate) endpoints: Vec<String>,
    pub(crate) gateway4: Option<String>,
    pub(crate) gateway6: Option<String>,
    pub(crate) auto_route4: bool,
    pub(crate) auto_route6: bool,
}

impl Clone for EndpointRouter {
    fn clone(&self) -> Self {
        Self {
            endpoints: self.endpoints.clone(),
            gateway4: self.gateway4.clone(),
            gateway6: self.gateway6.clone(),
            auto_route4: self.auto_route4,
            auto_route6: self.auto_route6,
        }
    }
}
impl Default for EndpointRouter {
    fn default() -> Self {
        EndpointRouter {
            endpoints: Vec::new(),
            gateway4: None,
            gateway6: None,
            auto_route4: false,
            auto_route6: false,
        }
    }
}

pub struct DnsManager {
    pub(crate) service_dns: HashMap<String, String>,
    pub(crate) service_dns_search: HashMap<String, String>,
}

impl Clone for DnsManager {
    fn clone(&self) -> Self {
        Self {
            service_dns: self.service_dns.clone(),
            service_dns_search: self.service_dns_search.clone(),
        }
    }
}

impl Default for DnsManager {
    fn default() -> Self {
        DnsManager {
            service_dns: Default::default(),
            service_dns_search: Default::default(),
        }
    }
}

pub struct TunnelManager {
    config: Config,
    real_interface: Option<String>,
    endpoint_router: EndpointRouter,
    dns_manager: DnsManager
}

impl TunnelManager {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            real_interface: None,
            endpoint_router: Default::default(),
            dns_manager: Default::default()
        }
    }

    fn interface_name(&self) -> &str {
        &self.config.network.identifier
    }

    pub fn start_tunnel(&mut self) -> TunnelResult<String> {
        if self.interface_exists()? {
            return Err(TunnelError::InterfaceExists(self.interface_name().to_string()));
        }

        self.add_interface()?;
        self.execute_hooks(HookType::PreUp)?;
        self.set_config()?;
        self.add_addresses()?;
        self.set_mtu_and_up()?;
        self.add_routes()?;
        self.set_endpoint_direct_route()?;
        self.set_dns()?;
        #[cfg(target_os = "macos")]
        {
            let iface = self.real_interface.as_ref().unwrap();
            let interface = self.interface_name();
            let this_peer = self.config.network.peers.get(&self.config.network.this_peer)
                .ok_or_else(|| TunnelError::InvalidConfig("This peer not found".to_string()))?;

            wg_quick_platform::start_monitor_daemon(iface, interface, &this_peer.dns, &this_peer.mtu, &self.endpoint_router, &self.dns_manager)?;
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

        match wg_quick_platform::interface_exists(interface) {
            Ok(Some(iface)) => {
                self.real_interface = Some(iface);
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(e) => Err(e),
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

    pub fn set_dns(&mut self) -> TunnelResult<()> {
        let this_peer = self.config.network.peers.get(&self.config.network.this_peer)
            .ok_or_else(|| TunnelError::InvalidConfig("This peer not found".to_string()))?;

        if !this_peer.dns.enabled || this_peer.dns.value.is_empty() {
            return Ok(());
        }

        let dns_servers = this_peer.dns.value.split(',').map(|s| s.trim().to_string()).collect();
        let interface_name = self.interface_name().to_string();
        wg_quick_platform::set_dns(&dns_servers, &interface_name, &mut self.dns_manager)
    }

    fn del_dns(&mut self) -> TunnelResult<()> {
        let interface_name = self.interface_name().to_string();
        wg_quick_platform::del_dns(&interface_name, &mut self.dns_manager)
    }

    fn add_routes(&mut self) -> TunnelResult<()> {
        let iface = self.real_interface.as_ref().unwrap();
        let allowed_ips = get_allowed_ips(iface)?;

        for cidr in allowed_ips {
            wg_quick_platform::add_route(iface, &self.config.network.identifier, &cidr, &mut self.endpoint_router)?;
        }

        Ok(())
    }

    fn set_endpoint_direct_route(&mut self) -> TunnelResult<()> {
        let iface = self.real_interface.as_ref().unwrap();
        wg_quick_platform::set_endpoint_direct_route(iface, &mut self.endpoint_router)
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
    let output = match cmd(&["wg", "show", iface, "allowed-ips"]) {
        Ok(output) => output,
        Err(e) => {
            log::warn!("Failed to get allowed IPs: {}, defaulting to an empty list of allowed IPs", e);
            return Ok(Vec::new());
        }
    };

    // Parse and collect valid CIDR entries
    let mut cidrs: Vec<String> = String::from_utf8_lossy(&*output.stdout)
        .split_whitespace()
        .filter(|s| wg_quickrs_wasm::validation::is_cidr(s))
        .map(String::from)
        .collect();

    // Sort by prefix length (descending)
    cidrs.sort_by(|a, b| {
        let prefix_a = a.split('/').nth(1).and_then(|p| p.parse::<u8>().ok()).unwrap_or(0);
        let prefix_b = b.split('/').nth(1).and_then(|p| p.parse::<u8>().ok()).unwrap_or(0);
        prefix_b.cmp(&prefix_a)
    });

    Ok(cidrs)
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
