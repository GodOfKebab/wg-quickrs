use std::collections::HashMap;
use std::process::{Command, Output};
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;
use thiserror::Error;

use wg_quickrs_wasm::types::{Config, EnabledValue, Peer};
use crate::macros::full_version;
use crate::wireguard::cmd::WireGuardCommandError;
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
    #[error("{0}")]
    WireGuardLibError(#[from] wg_quickrs_wasm::types::WireGuardLibError),
    #[error("{0}")]
    WireGuardCommandError(#[from] WireGuardCommandError),
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
    pub(crate) have_set_firewall: bool,
}

impl Clone for EndpointRouter {
    fn clone(&self) -> Self {
        Self {
            endpoints: self.endpoints.clone(),
            gateway4: self.gateway4.clone(),
            gateway6: self.gateway6.clone(),
            auto_route4: self.auto_route4.clone(),
            auto_route6: self.auto_route6.clone(),
            have_set_firewall: self.have_set_firewall.clone(),
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
            have_set_firewall: false,
        }
    }
}

pub struct DnsManager {
    pub(crate) have_set_dns: bool,
    pub(crate) service_dns: HashMap<String, String>,
    pub(crate) service_dns_search: HashMap<String, String>,
}

impl Clone for DnsManager {
    fn clone(&self) -> Self {
        Self {
            have_set_dns: self.have_set_dns.clone(),
            service_dns: self.service_dns.clone(),
            service_dns_search: self.service_dns_search.clone(),
        }
    }
}

impl Default for DnsManager {
    fn default() -> Self {
        DnsManager {
            have_set_dns: false,
            service_dns: Default::default(),
            service_dns_search: Default::default(),
        }
    }
}

pub struct TunnelManager {
    pub(crate) config: Option<Config>,
    real_interface: Option<String>,
    endpoint_router: EndpointRouter,
    dns_manager: DnsManager
}

impl TunnelManager {
    pub fn new(config: Option<Config>) -> Self {
        Self {
            config,
            real_interface: None,
            endpoint_router: Default::default(),
            dns_manager: Default::default()
        }
    }

    fn interface_name(&self) -> String {
        let config = self.config.as_ref().unwrap();
        config.network.identifier.clone()
    }

    fn this_peer(&self) -> TunnelResult<Peer> {
        let config = self.config.as_ref().unwrap();

        let this_peer = config.network.peers.get(&config.network.this_peer)
            .ok_or_else(|| TunnelError::InvalidConfig("This peer not found".to_string()))?;

        Ok(this_peer.clone())
    }

    pub fn start_tunnel(&mut self) -> TunnelResult<String> {
        let config = self.config
            .clone()
            .ok_or_else(|| WireGuardCommandError::Other("WireGuard config not initialized".to_string()))?;

        if self.interface_exists()? {
            return Err(TunnelError::InterfaceExists(self.interface_name()));
        }

        let interface = self.interface_name();
        log::info!("Starting WireGuard tunnel: {}", &interface);

        self.add_interface()?;
        self.execute_hooks(HookType::PreUp)?;
        self.set_config()?;
        self.add_addresses()?;
        self.set_mtu_and_up()?;
        self.add_routes()?;
        #[cfg(target_os = "macos")]
        {
            log::info!("[#] Setting endpoint direct route to WireGuard interface: {}", self.interface_name());
            let iface = self.real_interface.as_ref().unwrap();
            wg_quick_platform::set_endpoint_direct_route(iface, &mut self.endpoint_router)?;
        }
        self.set_dns()?;
        #[cfg(target_os = "macos")]
        {
            let iface = self.real_interface.as_ref().unwrap();
            let this_peer = &self.this_peer()?;

            wg_quick_platform::start_monitor_daemon(iface, &interface, &this_peer.dns, &this_peer.mtu, &self.endpoint_router, &self.dns_manager)?;
        }
        self.execute_hooks(HookType::PostUp)?;

        log::info!(
                "Started WireGuard tunnel at {}:{} (interface: {})",
                &config.agent.web.address,
                &config.agent.vpn.port,
                self.real_interface.clone().unwrap()
            );
        Ok(self.real_interface.clone().unwrap())
    }

    pub fn stop_tunnel(&mut self) -> TunnelResult<()> {
        let _ = self.config
            .clone()
            .ok_or_else(|| WireGuardCommandError::Other("WireGuard config not initialized".to_string()))?;

        if !self.interface_exists()? {
            log::warn!("Interface already deleted, skipping cleanup");
            return Ok(());
        }

        let interface = self.interface_name();
        log::info!("Stopping WireGuard tunnel: {}", &interface);

        if !self.is_wireguard_interface()? {
            return Err(TunnelError::InvalidConfig(format!(
                "'{}' is not a WireGuard interface",
                self.interface_name()
            )));
        }

        let _ = self.execute_hooks(HookType::PreDown);
        let _ = self.del_interface();
        let _ = self.del_routes();
        let _ = self.del_dns();
        let _ = self.execute_hooks(HookType::PostDown);

        log::info!("WireGuard tunnel stopped successfully");
        Ok(())
    }

    fn interface_exists(&mut self) -> TunnelResult<bool> {
        let interface = self.interface_name();

        match wg_quick_platform::interface_exists(&interface) {
            Ok(Some(iface)) => {
                log::info!("[#] Interface for {} is {}", &interface, iface);
                self.real_interface = Some(iface);
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn add_interface(&mut self) -> TunnelResult<()> {
        let interface = self.interface_name();
        log::info!("[#] Adding WireGuard interface: {}", &interface);
        self.real_interface = Some(wg_quick_platform::add_interface(&interface)?);
        Ok(())
    }

    fn del_interface(&mut self) -> TunnelResult<()> {
        let iface = self.real_interface.as_ref().ok_or_else(|| {
            TunnelError::InterfaceNotFound("No interface to delete".to_string())
        })?;
        let interface = self.interface_name();
        log::info!("[#] Deleting WireGuard interface: {}", &interface);

        let mut dns_manager = self.dns_manager.clone();
        let mut endpoint_router = self.endpoint_router.clone();
        wg_quick_platform::del_interface(iface, &interface, &mut dns_manager, &mut endpoint_router)?;
        self.dns_manager = dns_manager.clone();
        self.endpoint_router = endpoint_router.clone();

        Ok(())
    }

    fn add_addresses(&self) -> TunnelResult<()> {
        log::info!("[#] Adding addresses to WireGuard interface: {}", self.interface_name());
        let iface = self.real_interface.as_ref().unwrap();

        let this_peer = &self.this_peer()?;
        let addresses: Vec<&str> = this_peer.address.split(',').map(|s| s.trim()).collect();

        let config = self.config.as_ref().unwrap();
        let subnet_slash = config.network.subnet.as_str().split('/').nth(1).unwrap();

        for addr in addresses {
            let addr_w_subnet = format!("{}/{}", addr, subnet_slash);
            let is_ipv6 = addr_w_subnet.contains(':');
            wg_quick_platform::add_address(iface, &addr_w_subnet, is_ipv6)?;
        }
        Ok(())
    }

    fn set_mtu_and_up(&self) -> TunnelResult<()> {
        log::info!("[#] Setting MTU and bringing up WireGuard interface: {}", self.interface_name());
        let iface = self.real_interface.as_ref().unwrap();

        wg_quick_platform::set_mtu_and_up(iface, &self.this_peer()?.mtu)?;

        Ok(())
    }

    fn set_dns(&mut self) -> TunnelResult<()> {
        log::info!("[#] Setting DNS for WireGuard interface: {}", self.interface_name());
        let this_peer = &self.this_peer()?;

        if !this_peer.dns.enabled || this_peer.dns.value.is_empty() {
            return Ok(());
        }

        let dns_servers = this_peer.dns.value.split(',').map(|s| s.trim().to_string()).collect();
        let interface_name = self.interface_name();
        let _ = wg_quick_platform::set_dns(&dns_servers, &interface_name, &mut self.dns_manager);
        Ok(())
    }

    fn del_dns(&mut self) -> TunnelResult<()> {
        log::info!("[#] Deleting DNS for WireGuard interface: {}", self.interface_name());
        let interface_name = self.interface_name();
        wg_quick_platform::del_dns(&interface_name, &mut self.dns_manager)
    }

    fn add_routes(&mut self) -> TunnelResult<()> {
        log::info!("[#] Adding routes to WireGuard interface: {}", self.interface_name());
        let iface = self.real_interface.as_ref().unwrap();
        let allowed_ips = get_allowed_ips(iface)?;
        let config = self.config.as_ref().unwrap();

        for cidr in allowed_ips {
            wg_quick_platform::add_route(iface, &config.network.identifier, &cidr, &mut self.endpoint_router)?;
        }

        Ok(())
    }

    fn del_routes(&self) -> TunnelResult<()> {
        log::info!("[#] Deleting routes from WireGuard interface: {}", self.interface_name());
        let iface = self.real_interface.as_ref().ok_or_else(|| {
            TunnelError::InterfaceNotFound("No interface for route deletion".to_string())
        })?;

        wg_quick_platform::del_routes(iface)
    }

    fn set_config(&self) -> TunnelResult<()> {
        log::info!("[#] Setting WireGuard interface configuration: {}", self.interface_name());
        let iface = self.real_interface.as_ref().unwrap();
        let config = self.config.as_ref().unwrap();

        let wg_config = wg_quickrs_wasm::helpers::get_peer_wg_config(&config.network, &config.network.this_peer, full_version!(), true, None)?;

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

        Ok(interfaces.contains(&&*interface_name) ||
            (real_iface.is_some() && interfaces.contains(&real_iface.unwrap())))
    }

    fn execute_hooks(&self, hook_type: HookType) -> TunnelResult<()> {
        log::info!("[#] Executing {:?} hooks", hook_type);
        let this_peer = &self.this_peer()?;
        let config = self.config.as_ref().unwrap();

        let fw_utility = &config.agent.firewall.utility.to_string_lossy();
        let subnet = &config.network.subnet;
        let gateway = &config.agent.firewall.gateway;
        let port = &config.agent.vpn.port;
        let interface = &config.network.identifier;

        let mut cmds: Vec<EnabledValue> = Vec::new();
        let hooks = match hook_type {
            HookType::PreUp => &this_peer.scripts.pre_up,
            HookType::PostUp => {
                if config.agent.firewall.enabled && let Some(utility) = config.agent.firewall.utility.file_name() {
                    if utility == "iptables" {
                        let _ = cmd(&[fw_utility, "-t", "nat", "-I", "POSTROUTING", "-s", subnet, "-o", gateway, "-j", "MASQUERADE"]);
                        let _ = cmd(&[fw_utility, "-I", "INPUT", "-p", "udp", "-m", "udp", "--dport", &port.to_string(), "-j", "ACCEPT"]);
                        let _ = cmd(&[fw_utility, "-I", "FORWARD", "-i", interface, "-j", "ACCEPT"]);
                        let _ = cmd(&[fw_utility, "-I", "FORWARD", "-o", interface, "-j", "ACCEPT"]);
                        #[cfg(not(feature = "docker"))]
                        let _ = cmd(&["sysctl", "-w", "net.ipv4.ip_forward=1"]);
                    } else if utility == "pfctl" {
                        match mod_pf_conf(&config.agent.firewall.gateway, &config.network.subnet, true) {
                            Ok(()) => {
                                let _ = cmd(&[fw_utility, "-f", "/etc/pf.conf"]);
                                let _ = cmd(&[fw_utility, "-e"]);
                                let _ = cmd(&["sysctl", "-w", "net.inet.ip.forwarding=1"]);
                            },
                            Err(e) => log::warn!("Warning: Failed to modify pf.conf: {}", e),
                        }
                    }
                }
                cmds.extend(this_peer.scripts.post_up.clone());
                &cmds
            },
            HookType::PreDown => &this_peer.scripts.pre_down,
            HookType::PostDown => {
                if config.agent.firewall.enabled && let Some(utility) = config.agent.firewall.utility.file_name() {
                    if utility == "iptables" {
                        let _ = cmd(&[fw_utility, "-t", "nat", "-D", "POSTROUTING", "-s", subnet, "-o", gateway, "-j", "MASQUERADE"]);
                        let _ = cmd(&[fw_utility, "-D", "INPUT", "-p", "udp", "-m", "udp", "--dport", &port.to_string(), "-j", "ACCEPT"]);
                        let _ = cmd(&[fw_utility, "-D", "FORWARD", "-i", interface, "-j", "ACCEPT"]);
                        let _ = cmd(&[fw_utility, "-D", "FORWARD", "-o", interface, "-j", "ACCEPT"]);
                        #[cfg(not(feature = "docker"))]
                        let _ = cmd(&["sysctl", "-w", "net.ipv4.ip_forward=0"]);
                    } else if utility == "pfctl" {
                        match mod_pf_conf(&config.agent.firewall.gateway, &config.network.subnet, false) {
                            Ok(_) => {}
                            Err(e) => log::warn!("Warning: Failed to modify pf.conf: {}", e),
                        };
                        let _ = cmd(&[fw_utility, "-f", "/etc/pf.conf"]);
                        let _ = cmd(&[fw_utility, "-d"]);
                        #[cfg(not(feature = "docker"))]
                        let _ = cmd(&["sysctl", "-w", "net.inet.ip.forwarding=0"]);
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

fn extract_ip_from_endpoint(endpoint: &str) -> Option<String> {
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

fn get_allowed_ips(iface: &str) -> TunnelResult<Vec<String>> {
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

pub fn cmd(args: &[&str]) -> TunnelResult<Output> {
    if args.is_empty() {
        return Err(TunnelError::CommandFailed("Empty command".to_string()));
    }

    log::info!("[+] {}", args.join(" "));

    let output = Command::new(args[0])
        .args(&args[1..])
        .output()?;
    if !output.stderr.is_empty() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if !output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
        return Err(TunnelError::CommandFailed(String::from_utf8_lossy(&output.stderr).to_string()));
    }

    Ok(output)
}

fn mod_pf_conf(gateway: &str, subnet: &str, add: bool) -> TunnelResult<()> {
    let nat_rule = format!("nat on {gateway} from {subnet} to any -> {gateway}  # added by wg-quickrs");

    let pf_conf_path = "/etc/pf.conf";
    let pf_conf_new = "/etc/pf.conf.new";
    let pf_conf_bak = "/etc/pf.conf.bak";

    // Read the file
    let content = fs::read_to_string(pf_conf_path)
        .map_err(|e| TunnelError::IoError(e))?;

    // Check if the rule already exists
    let rule_exists = content.lines().any(|line| line == nat_rule);

    if add {
        // Adding rule
        log::info!("*** adding the nat rule to pf.conf...");
        if rule_exists {
            log::info!("*** already exists, nothing to do");
            return Ok(()); // Already exists, nothing to do
        }

        // Build new content with rule inserted after the first "nat" line
        let mut new_lines = Vec::new();
        let mut found_nat = false;

        for line in content.lines() {
            new_lines.push(line.to_string());

            if !found_nat && line.starts_with("nat") {
                found_nat = true;
                new_lines.push(nat_rule.to_string());
            }
        }

        // Check if the NAT section was found
        if !found_nat {
            eprintln!("*** could NOT configure firewall because there are no existing NAT rules. See notes at docs/MACOS-FIREWALL.md");
            return Err(TunnelError::InvalidConfig(
                "No existing NAT rules in /etc/pf.conf".into()
            ));
        }

        // Write to a temporary file
        fs::write(pf_conf_new, new_lines.join("\n") + "\n")?;
        log::info!("*** added the nat rule to pf.conf");
    } else {
        // Removing rule
        log::info!("*** removing the nat rule from pf.conf...");
        if !rule_exists {
            log::info!("*** already removed, nothing to do");
            return Ok(()); // Doesn't exist, nothing to do
        }

        // Filter out the rule
        let new_lines: Vec<String> = content
            .lines()
            .filter(|line| *line != nat_rule)
            .map(|s| s.to_string())
            .collect();

        // Write to a temporary file
        fs::write(pf_conf_new, new_lines.join("\n") + "\n")?;
        log::info!("*** removed the nat rule from pf.conf");
    }

    // Atomic operations: backup then replace
    if std::path::Path::new(pf_conf_bak).exists() {
        fs::remove_file(pf_conf_bak)?;
    }
    fs::rename(pf_conf_path, pf_conf_bak)?;
    fs::rename(pf_conf_new, pf_conf_path)?;

    Ok(())
}
