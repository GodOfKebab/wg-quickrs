use std::collections::HashMap;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;
use thiserror::Error;
use wg_quickrs_lib::types::config::Config;
use wg_quickrs_lib::types::network::{Peer};
use crate::helpers::{shell_cmd, ShellError};
#[cfg(target_os = "macos")]
use crate::wireguard::wg_quick_darwin as wg_quick_platform;
#[cfg(target_os = "linux")]
use crate::wireguard::wg_quick_linux as wg_quick_platform;


#[derive(Error, Debug)]
pub enum TunnelError {
    #[error("WireGuard config not initialized")]
    ConfigNotInitialized(),
    #[error("command execution failed: {0}")]
    CommandFailed(String),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("interface already exists: {0}")]
    InterfaceExists(String),
    #[error("interface not found: {0}")]
    InterfaceNotFound(String),
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("{0}")]
    WireGuardLibError(#[from] wg_quickrs_lib::types::misc::WireGuardLibError),
    #[error("{0}")]
    ShellError(#[from] ShellError),
    #[cfg(target_os = "macos")]
    #[error("unable to find default gateway")]
    DefaultGatewayNotFound(),
}

pub type TunnelResult<T> = Result<T, TunnelError>;

#[allow(dead_code)]
#[derive(Default)]
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
            auto_route4: self.auto_route4,
            auto_route6: self.auto_route6,
            have_set_firewall: self.have_set_firewall,
        }
    }
}

#[derive(Default)]
pub struct DnsManager {
    pub(crate) have_set_dns: bool,
    pub(crate) service_dns: HashMap<String, String>,
    pub(crate) service_dns_search: HashMap<String, String>,
}

impl Clone for DnsManager {
    fn clone(&self) -> Self {
        Self {
            have_set_dns: self.have_set_dns,
            service_dns: self.service_dns.clone(),
            service_dns_search: self.service_dns_search.clone(),
        }
    }
}


pub struct TunnelManager {
    pub(crate) config: Option<Config>,
    pub(crate) real_interface: Option<String>,
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
        config.network.name.clone()
    }

    fn this_peer(&self) -> TunnelResult<Peer> {
        let config = self.config.as_ref().unwrap();

        let this_peer = config.network.peers.get(&config.network.this_peer)
            .ok_or_else(|| TunnelError::InvalidConfig("This peer not found".to_string()))?;

        Ok(this_peer.clone())
    }

    pub fn start_tunnel(&mut self) -> TunnelResult<()> {
        let config = self.config
            .clone()
            .ok_or_else(|| TunnelError::ConfigNotInitialized())?;

        if self.interface_exists()? {
            return Err(TunnelError::InterfaceExists(self.interface_name()));
        }

        let interface = self.interface_name();
        log::info!("Starting WireGuard tunnel: {}...", &interface);

        self.add_interface()?;
        self.execute_hooks(HookType::PreUp)?;
        self.set_config()?;
        self.add_addresses()?;
        self.set_mtu_and_up()?;
        self.add_routes()?;
        #[cfg(target_os = "macos")]
        {
            log::debug!("[#] Setting endpoint direct route to WireGuard interface: {}", self.interface_name());
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
                "Started WireGuard tunnel at *:{} (wg interface: {})",
                &config.agent.vpn.port,
                self.real_interface.clone().unwrap()
            );
        Ok(())
    }

    pub fn stop_tunnel(&mut self) -> TunnelResult<()> {
        let _ = self.config
            .clone()
            .ok_or_else(|| TunnelError::ConfigNotInitialized())?;

        if !self.interface_exists()? {
            log::debug!("Interface already deleted, skipping cleanup");
            return Ok(());
        }

        let interface = self.interface_name();
        log::info!("Stopping WireGuard tunnel: {}...", &interface);

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
                log::debug!("[#] Interface for {} is {}", &interface, iface);
                self.real_interface = Some(iface);
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn add_interface(&mut self) -> TunnelResult<()> {
        let interface = self.interface_name();
        log::debug!("[#] Adding WireGuard interface: {}", &interface);
        self.real_interface = Some(wg_quick_platform::add_interface(&interface)?);
        Ok(())
    }

    fn del_interface(&mut self) -> TunnelResult<()> {
        let iface = self.real_interface.as_ref().ok_or_else(|| {
            TunnelError::InterfaceNotFound("No interface to delete".to_string())
        })?;
        let interface = self.interface_name();
        log::debug!("[#] Deleting WireGuard interface: {}", &interface);

        let mut dns_manager = self.dns_manager.clone();
        let mut endpoint_router = self.endpoint_router.clone();
        wg_quick_platform::del_interface(iface, &interface, &mut dns_manager, &mut endpoint_router)?;
        self.dns_manager = dns_manager.clone();
        self.endpoint_router = endpoint_router.clone();

        Ok(())
    }

    fn add_addresses(&self) -> TunnelResult<()> {
        log::debug!("[#] Adding addresses to WireGuard interface: {}", self.interface_name());
        let iface = self.real_interface.as_ref().unwrap();

        let this_peer = &self.this_peer()?;
        let addresses = vec![this_peer.address];

        let config = self.config.as_ref().unwrap();
        let subnet_slash = config.network.subnet.prefix_len();

        for addr in addresses {
            let addr_w_subnet = format!("{}/{}", addr, subnet_slash);
            let is_ipv6 = addr_w_subnet.contains(':');
            wg_quick_platform::add_address(iface, &addr_w_subnet, is_ipv6)?;
        }
        Ok(())
    }

    fn set_mtu_and_up(&self) -> TunnelResult<()> {
        log::debug!("[#] Setting MTU and bringing up WireGuard interface: {}", self.interface_name());
        let iface = self.real_interface.as_ref().unwrap();

        wg_quick_platform::set_mtu_and_up(iface, &self.this_peer()?.mtu)?;

        Ok(())
    }

    fn set_dns(&mut self) -> TunnelResult<()> {
        log::debug!("[#] Setting DNS for WireGuard interface: {}", self.interface_name());
        let this_peer = &self.this_peer()?;

        if !this_peer.dns.enabled || this_peer.dns.addresses.is_empty() {
            return Ok(());
        }

        let dns_servers = this_peer.dns.addresses.clone();
        let interface_name = self.interface_name();
        let _ = wg_quick_platform::set_dns(&dns_servers, &interface_name, &mut self.dns_manager);
        Ok(())
    }

    fn del_dns(&mut self) -> TunnelResult<()> {
        log::debug!("[#] Deleting DNS for WireGuard interface: {}", self.interface_name());
        let interface_name = self.interface_name();
        wg_quick_platform::del_dns(&interface_name, &mut self.dns_manager)
    }

    fn add_routes(&mut self) -> TunnelResult<()> {
        log::debug!("[#] Adding routes to WireGuard interface: {}", self.interface_name());
        let iface = self.real_interface.as_ref().unwrap();
        let allowed_ips = get_allowed_ips(iface)?;
        let config = self.config.as_ref().unwrap();

        for cidr in allowed_ips {
            wg_quick_platform::add_route(iface, &config.network.name, &cidr, &mut self.endpoint_router)?;
        }

        Ok(())
    }

    fn del_routes(&self) -> TunnelResult<()> {
        log::debug!("[#] Deleting routes from WireGuard interface: {}", self.interface_name());
        let iface = self.real_interface.as_ref().ok_or_else(|| {
            TunnelError::InterfaceNotFound("No interface for route deletion".to_string())
        })?;

        wg_quick_platform::del_routes(iface)
    }

    fn set_config(&self) -> TunnelResult<()> {
        log::debug!("[#] Setting WireGuard interface configuration: {}", self.interface_name());
        let iface = self.real_interface.as_ref().unwrap();
        let config = self.config.as_ref().unwrap();

        let wg_config = wg_quickrs_lib::helpers::get_peer_wg_config(&config.network, &config.network.this_peer, true)?;

        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "{}", wg_config)?;
        shell_cmd(&["wg", "setconf", iface, &temp_file.path().to_string_lossy()])?;
        let _ = fs::remove_file(&temp_file);

        Ok(())
    }

    fn is_wireguard_interface(&self) -> TunnelResult<bool> {
        let output = shell_cmd(&["wg", "show", "interfaces"])?;

        if !output.status.success() {
            return Ok(false);
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let interfaces: Vec<&str> = output_str.split_whitespace().collect();

        let interface_name = self.interface_name();
        let real_iface = self.real_interface.as_deref();

        Ok(interfaces.contains(&&*interface_name) ||
            (real_iface.is_some() && interfaces.contains(&real_iface.unwrap())))
    }

    fn execute_hooks(&self, hook_type: HookType) -> TunnelResult<()> {
        log::debug!("[#] Executing WireGuard {:?} hooks", hook_type);
        let this_peer = &self.this_peer()?;
        let config = self.config.as_ref().unwrap();

        // Execute VPN firewall scripts
        if config.agent.vpn.enabled {
            let vpn_hooks = match hook_type {
                HookType::PreUp => &config.agent.firewall.vpn.pre_up,
                HookType::PostUp => &config.agent.firewall.vpn.post_up,
                HookType::PreDown => &config.agent.firewall.vpn.pre_down,
                HookType::PostDown => &config.agent.firewall.vpn.post_down,
            };

            for hook in vpn_hooks {
                if !hook.enabled {
                    continue;
                }

                let output = shell_cmd(&["sh", "-c", &hook.script])?;
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    log::warn!("Warning: VPN firewall script failed: {}", stderr);
                }
            }
        }

        // Execute peer scripts
        let peer_hooks = match hook_type {
            HookType::PreUp => &this_peer.scripts.pre_up,
            HookType::PostUp => &this_peer.scripts.post_up,
            HookType::PreDown => &this_peer.scripts.pre_down,
            HookType::PostDown => &this_peer.scripts.post_down,
        };

        for hook in peer_hooks {
            if !hook.enabled {
                continue;
            }

            let output = shell_cmd(&["sh", "-c", &hook.script])?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log::warn!("Warning: Hook failed: {}", stderr);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum HookType {
    PreUp,
    PostUp,
    PreDown,
    PostDown,
}

fn extract_ip_from_endpoint(endpoint: &str) -> Option<String> {
    if endpoint.starts_with('[')
        && let Some(end) = endpoint.find(']') {
            return Some(endpoint[1..end].to_string());
        }

    if let Some(colon_pos) = endpoint.rfind(':') {
        return Some(endpoint[..colon_pos].to_string());
    }

    None
}

fn get_allowed_ips(iface: &str) -> TunnelResult<Vec<String>> {
    let output = match shell_cmd(&["wg", "show", iface, "allowed-ips"]) {
        Ok(output) => output,
        Err(e) => {
            log::warn!("Failed to get allowed IPs: {}, defaulting to an empty list of allowed IPs", e);
            return Ok(Vec::new());
        }
    };

    // Parse and collect valid CIDR entries
    let mut cidrs: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .filter(|s| wg_quickrs_lib::validation::network::parse_and_validate_ipv4_subnet(s).is_ok())
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
    let output = match shell_cmd(&["wg", "show", iface, "endpoints"]) {
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
        if parts.len() > 1
            && let Some(ip) = extract_ip_from_endpoint(parts[1]) {
                endpoints.push(ip.clone());
            }
    }

    endpoints
}
