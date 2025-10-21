use std::process::Command;
use std::fs;
use std::io::Write;
use thiserror::Error;

use wg_quickrs_wasm::types::{Config, EnabledValue};
use wg_quickrs_wasm::helpers::{wg_public_key_from_private_key, get_connection_id};

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
}

type Result<T> = std::result::Result<T, TunnelError>;

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

    pub fn start_tunnel(&mut self) -> Result<String> {
        if self.interface_exists()? {
            return Err(TunnelError::InterfaceExists(self.interface_name().to_string()));
        }

        self.execute_hooks(HookType::PreUp)?;
        self.add_interface()?;
        self.set_config()?;
        self.add_addresses()?;
        self.set_mtu_and_up()?;
        self.add_routes()?;

        #[cfg(target_os = "macos")]
        self.start_monitor_daemon()?;

        self.execute_hooks(HookType::PostUp)?;

        Ok(self.real_interface.clone().unwrap_or_else(|| self.interface_name().to_string()))
    }

    pub fn stop_tunnel(&mut self) -> Result<()> {
        let _ = self.interface_exists();

        if !self.is_wireguard_interface()? {
            return Err(TunnelError::InvalidConfig(format!(
                "'{}' is not a WireGuard interface",
                self.interface_name()
            )));
        }

        self.execute_hooks(HookType::PreDown)?;
        self.del_routes()?;
        self.del_dns()?;
        self.del_interface()?;
        self.execute_hooks(HookType::PostDown)?;

        Ok(())
    }

    fn interface_exists(&mut self) -> Result<bool> {
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
            let output = Command::new("ip")
                .args(&["link", "show", "dev", interface])
                .output()?;

            if output.status.success() {
                self.real_interface = Some(interface.to_string());
            }
            Ok(output.status.success())
        }
    }

    fn add_interface(&mut self) -> Result<()> {
        let interface = self.interface_name();

        #[cfg(target_os = "macos")]
        {
            fs::create_dir_all("/var/run/wireguard/")?;

            let name_file = format!("/var/run/wireguard/{}.name", interface);
            let script = format!(
                "export WG_TUN_NAME_FILE='{}' && wireguard-go utun",
                name_file
            );

            self.cmd(&["bash", "-c", &script])?;
            let iface = fs::read_to_string(&name_file)?.trim().to_string();
            self.real_interface = Some(iface);
        }

        #[cfg(target_os = "linux")]
        {
            let result = Command::new("ip")
                .args(&["link", "add", "dev", interface, "type", "wireguard"])
                .output()?;

            if !result.status.success() {
                let userspace_result = Command::new("wireguard-go")
                    .arg(interface)
                    .output()?;

                if !userspace_result.status.success() {
                    return Err(TunnelError::CommandFailed(
                        "Failed to create WireGuard interface".to_string()
                    ));
                }
            }
            self.real_interface = Some(interface.to_string());
        }

        Ok(())
    }

    fn del_interface(&mut self) -> Result<()> {
        let iface = self.real_interface.as_ref().ok_or_else(|| {
            TunnelError::InterfaceNotFound("No interface to delete".to_string())
        })?;
        let interface = self.interface_name();

        #[cfg(target_os = "macos")]
        {
            let sock_file = format!("/var/run/wireguard/{}.sock", iface);
            let _ = fs::remove_file(sock_file);

            let name_file = format!("/var/run/wireguard/{}.name", interface);
            let _ = fs::remove_file(name_file);
        }

        #[cfg(target_os = "linux")]
        {
            self.cmd(&["ip", "link", "delete", "dev", iface])?;
        }

        Ok(())
    }

    fn add_addresses(&self) -> Result<()> {
        let iface = self.real_interface.as_ref().unwrap();

        let this_peer = self.config.network.peers.get(&self.config.network.this_peer)
            .ok_or_else(|| TunnelError::InvalidConfig("This peer not found".to_string()))?;

        let addresses: Vec<&str> = this_peer.address.split(',').map(|s| s.trim()).collect();

        for addr in addresses {
            let is_ipv6 = addr.contains(':');

            #[cfg(target_os = "macos")]
            {
                if is_ipv6 {
                    self.cmd(&["ifconfig", iface, "inet6", addr, "alias"])?;
                } else {
                    let ip = addr.split('/').next().unwrap();
                    self.cmd(&["ifconfig", iface, "inet", addr, ip, "alias"])?;
                }
            }

            #[cfg(target_os = "linux")]
            {
                let proto = if is_ipv6 { "-6" } else { "-4" };
                self.cmd(&["ip", proto, "address", "add", addr, "dev", iface])?;
            }
        }
        Ok(())
    }

    fn set_mtu_and_up(&self) -> Result<()> {
        let iface = self.real_interface.as_ref().unwrap();

        let this_peer = self.config.network.peers.get(&self.config.network.this_peer)
            .ok_or_else(|| TunnelError::InvalidConfig("This peer not found".to_string()))?;

        let mtu = if this_peer.mtu.enabled {
            this_peer.mtu.value.clone()
        } else {
            self.calculate_mtu().unwrap_or(1420).to_string()
        };

        #[cfg(target_os = "macos")]
        {
            self.cmd(&["ifconfig", iface, "mtu", &mtu])?;
            self.cmd(&["ifconfig", iface, "up"])?;
        }

        #[cfg(target_os = "linux")]
        {
            self.cmd(&["ip", "link", "set", "mtu", &mtu, "up", "dev", iface])?;
        }

        Ok(())
    }

    fn calculate_mtu(&self) -> Option<u16> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("netstat")
                .args(&["-nr", "-f", "inet"])
                .output()
                .ok()?;

            let output_str = String::from_utf8_lossy(&output.stdout);

            for line in output_str.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 5 && parts[0] == "default" {
                    let default_iface = parts[5];
                    let ifconfig_output = Command::new("ifconfig")
                        .arg(default_iface)
                        .output()
                        .ok()?;

                    let ifconfig_str = String::from_utf8_lossy(&ifconfig_output.stdout);
                    for line in ifconfig_str.lines() {
                        if line.contains("mtu") {
                            if let Some(mtu_str) = line.split("mtu").nth(1) {
                                if let Some(mtu) = mtu_str.trim().split_whitespace().next() {
                                    if let Ok(mtu_val) = mtu.parse::<u16>() {
                                        return Some(mtu_val.saturating_sub(80));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let endpoints = self.get_endpoints().ok()?;
            let mut min_mtu = 2147483647u32;

            for endpoint in endpoints {
                let output = Command::new("ip")
                    .args(&["route", "get", &endpoint])
                    .output()
                    .ok()?;

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

            return Some((min_mtu.saturating_sub(80)) as u16);
        }

        None
    }

    fn set_dns(&self) -> Result<()> {
        let this_peer = self.config.network.peers.get(&self.config.network.this_peer)
            .ok_or_else(|| TunnelError::InvalidConfig("This peer not found".to_string()))?;

        if !this_peer.dns.enabled || this_peer.dns.value.is_empty() {
            return Ok(());
        }

        let dns_servers: Vec<&str> = this_peer.dns.value.split(',').map(|s| s.trim()).collect();

        #[cfg(target_os = "macos")]
        {
            let output = Command::new("networksetup")
                .arg("-listallnetworkservices")
                .output()?;

            let services = String::from_utf8_lossy(&output.stdout);

            for service in services.lines().skip(1) {
                let service = service.trim_start_matches('*').trim();

                let mut args = vec!["-setdnsservers", service];
                args.extend(dns_servers.iter().copied());
                let _ = Command::new("networksetup").args(&args).output();
            }
        }

        #[cfg(target_os = "linux")]
        {
            let mut dns_config = String::new();
            for server in dns_servers {
                dns_config.push_str(&format!("nameserver {}\n", server));
            }

            let mut child = Command::new("resolvconf")
                .args(&["-a", self.interface_name(), "-m", "0", "-x"])
                .stdin(std::process::Stdio::piped())
                .spawn()?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(dns_config.as_bytes())?;
            }

            child.wait()?;
        }

        Ok(())
    }

    fn del_dns(&self) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("networksetup")
                .arg("-listallnetworkservices")
                .output()?;

            let services = String::from_utf8_lossy(&output.stdout);

            for service in services.lines().skip(1) {
                let service = service.trim_start_matches('*').trim();
                let _ = Command::new("networksetup")
                    .args(&["-setdnsservers", service, "Empty"])
                    .output();
            }
        }

        #[cfg(target_os = "linux")]
        {
            let _ = Command::new("resolvconf")
                .args(&["-d", self.interface_name(), "-f"])
                .output();
        }

        Ok(())
    }

    fn add_routes(&self) -> Result<()> {
        let iface = self.real_interface.as_ref().unwrap();
        let allowed_ips = self.get_allowed_ips()?;

        for ip in allowed_ips {
            let is_default = ip.ends_with("/0");
            let is_ipv6 = ip.contains(':');

            #[cfg(target_os = "macos")]
            {
                if is_default {
                    if is_ipv6 {
                        self.cmd(&["route", "-q", "-n", "add", "-inet6", "::/1", "-interface", iface])?;
                        self.cmd(&["route", "-q", "-n", "add", "-inet6", "8000::/1", "-interface", iface])?;
                    } else {
                        self.cmd(&["route", "-q", "-n", "add", "-inet", "0.0.0.0/1", "-interface", iface])?;
                        self.cmd(&["route", "-q", "-n", "add", "-inet", "128.0.0.0/1", "-interface", iface])?;
                    }

                    self.set_endpoint_direct_routes()?;
                } else {
                    let family = if is_ipv6 { "-inet6" } else { "-inet" };
                    let _ = self.cmd(&["route", "-q", "-n", "add", family, &ip, "-interface", iface]);
                }
            }

            #[cfg(target_os = "linux")]
            {
                let proto = if is_ipv6 { "-6" } else { "-4" };

                if is_default {
                    self.add_default_route(&ip, proto)?;
                } else {
                    let _ = self.cmd(&["ip", proto, "route", "add", &ip, "dev", iface]);
                }
            }
        }

        Ok(())
    }

    fn del_routes(&self) -> Result<()> {
        let iface = self.real_interface.as_ref().ok_or_else(|| {
            TunnelError::InterfaceNotFound("No interface for route deletion".to_string())
        })?;

        #[cfg(target_os = "macos")]
        {
            let output = Command::new("netstat")
                .args(&["-nr", "-f", "inet"])
                .output()?;

            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 5 && parts[5] == iface {
                    let dest = parts[0];
                    let _ = self.cmd(&["route", "-q", "-n", "delete", "-inet", dest]);
                }
            }

            let output = Command::new("netstat")
                .args(&["-nr", "-f", "inet6"])
                .output()?;

            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 3 && parts[3] == iface {
                    let dest = parts[0];
                    let _ = self.cmd(&["route", "-q", "-n", "delete", "-inet6", dest]);
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Routes are automatically cleaned up when interface is deleted
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn set_endpoint_direct_routes(&self) -> Result<()> {
        let gateway4 = self.get_default_gateway(false);
        let gateway6 = self.get_default_gateway(true);
        let endpoints = self.get_endpoints()?;

        for endpoint in endpoints {
            if endpoint.contains(':') && gateway6.is_some() {
                let gw = gateway6.as_ref().unwrap();
                let _ = self.cmd(&["route", "-q", "-n", "add", "-inet6", &endpoint, "-gateway", gw]);
            } else if gateway4.is_some() {
                let gw = gateway4.as_ref().unwrap();
                let _ = self.cmd(&["route", "-q", "-n", "add", "-inet", &endpoint, "-gateway", gw]);
            }
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn get_default_gateway(&self, ipv6: bool) -> Option<String> {
        let family = if ipv6 { "inet6" } else { "inet" };
        let output = Command::new("netstat")
            .args(&["-nr", "-f", family])
            .output()
            .ok()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 && parts[0] == "default" && !parts[1].starts_with("link#") {
                return Some(parts[1].to_string());
            }
        }
        None
    }

    #[cfg(target_os = "linux")]
    fn add_default_route(&self, route: &str, proto: &str) -> Result<()> {
        let iface = self.real_interface.as_ref().unwrap();

        let table = self.get_or_create_fwmark_table()?;

        self.cmd(&["ip", proto, "rule", "add", "not", "fwmark", &table.to_string(), "table", &table.to_string()])?;
        self.cmd(&["ip", proto, "rule", "add", "table", "main", "suppress_prefixlength", "0"])?;
        self.cmd(&["ip", proto, "route", "add", route, "dev", iface, "table", &table.to_string()])?;

        self.setup_firewall(table, proto)?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn get_or_create_fwmark_table(&self) -> Result<u32> {
        let iface = self.real_interface.as_ref().unwrap();
        let output = Command::new("wg")
            .args(&["show", iface, "fwmark"])
            .output()?;

        let fwmark_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if fwmark_str.is_empty() || fwmark_str == "off" {
            let mut table = 51820u32;
            loop {
                let v4_check = Command::new("ip")
                    .args(&["-4", "route", "show", "table", &table.to_string()])
                    .output()?;
                let v6_check = Command::new("ip")
                    .args(&["-6", "route", "show", "table", &table.to_string()])
                    .output()?;

                if v4_check.stdout.is_empty() && v6_check.stdout.is_empty() {
                    break;
                }
                table += 1;
            }

            self.cmd(&["wg", "set", iface, "fwmark", &table.to_string()])?;
            Ok(table)
        } else {
            fwmark_str.parse::<u32>()
                .map_err(|_| TunnelError::InvalidConfig("Invalid fwmark".to_string()))
        }
    }

    #[cfg(target_os = "linux")]
    fn setup_firewall(&self, table: u32, proto: &str) -> Result<()> {
        let pf = if proto == "-6" { "ip6" } else { "ip" };

        if Command::new("nft").arg("--version").output().is_ok() {
            let nftable = format!("wg-quick-{}", self.interface_name());
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

    fn set_config(&self) -> Result<()> {
        let iface = self.real_interface.as_ref().unwrap();
        let wg_config = self.generate_wg_config()?;

        let temp_file = format!("/tmp/wg-{}.conf", self.interface_name());
        fs::write(&temp_file, wg_config)?;

        let result = Command::new("wg")
            .args(&["setconf", iface, &temp_file])
            .output()?;

        let _ = fs::remove_file(&temp_file);

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(TunnelError::CommandFailed(stderr.to_string()));
        }

        Ok(())
    }

    fn generate_wg_config(&self) -> Result<String> {
        let mut config = String::new();
        let network = &self.config.network;

        let this_peer = network.peers.get(&network.this_peer)
            .ok_or_else(|| TunnelError::InvalidConfig("This peer not found".to_string()))?;

        config.push_str("[Interface]\n");
        config.push_str(&format!("PrivateKey = {}\n", this_peer.private_key));

        if this_peer.endpoint.enabled {
            if let Some((_host, port)) = this_peer.endpoint.value.rsplit_once(':') {
                config.push_str(&format!("ListenPort = {}\n", port));
            }
        }

        for (peer_id, peer) in &network.peers {
            if peer_id == &network.this_peer {
                continue;
            }

            let connection_id = get_connection_id(&network.this_peer, peer_id);
            let connection = network.connections.get(&connection_id);

            if let Some(conn) = connection {
                if !conn.enabled {
                    continue;
                }

                config.push_str("\n[Peer]\n");
                config.push_str(&format!("PublicKey = {}\n", wg_public_key_from_private_key(&peer.private_key)?));

                if peer.endpoint.enabled && !peer.endpoint.value.is_empty() {
                    config.push_str(&format!("Endpoint = {}\n", peer.endpoint.value));
                }

                let allowed_ips = if connection_id.starts_with(&format!("{}*", network.this_peer)) {
                    &conn.allowed_ips_a_to_b
                } else {
                    &conn.allowed_ips_b_to_a
                };

                if !allowed_ips.is_empty() {
                    config.push_str(&format!("AllowedIPs = {}\n", allowed_ips));
                }

                if !conn.pre_shared_key.is_empty() {
                    config.push_str(&format!("PresharedKey = {}\n", conn.pre_shared_key));
                }

                if conn.persistent_keepalive.enabled {
                    config.push_str(&format!("PersistentKeepalive = {}\n", conn.persistent_keepalive.value));
                }
            }
        }

        Ok(config)
    }

    fn get_allowed_ips(&self) -> Result<Vec<String>> {
        let iface = self.real_interface.as_ref().unwrap();
        let output = Command::new("wg")
            .args(&["show", iface, "allowed-ips"])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
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

    fn get_endpoints(&self) -> Result<Vec<String>> {
        let iface = self.real_interface.as_ref().unwrap();
        let output = Command::new("wg")
            .args(&["show", iface, "endpoints"])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

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

        Ok(endpoints)
    }

    fn is_wireguard_interface(&self) -> Result<bool> {
        let output = Command::new("wg")
            .args(&["show", "interfaces"])
            .output()?;

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

    fn execute_hooks(&self, hook_type: HookType) -> Result<()> {
        let this_peer = self.config.network.peers.get(&self.config.network.this_peer)
            .ok_or_else(|| TunnelError::InvalidConfig("This peer not found".to_string()))?;

        let hooks = match hook_type {
            HookType::PreUp => &this_peer.scripts.pre_up,
            HookType::PostUp => &this_peer.scripts.post_up,
            HookType::PreDown => &this_peer.scripts.pre_down,
            HookType::PostDown => &this_peer.scripts.post_down,
        };

        for hook in hooks {
            if !hook.enabled {
                continue;
            }

            let real_iface = self.real_interface.as_ref()
                .map(|s| s.as_str())
                .unwrap_or(self.interface_name());

            let script = hook.value
                .replace("%i", real_iface)
                .replace("%I", self.interface_name());

            eprintln!("[#] {}", script);

            let output = Command::new("bash")
                .arg("-c")
                .arg(&script)
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("Warning: Hook failed: {}", stderr);
            }
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn start_monitor_daemon(&self) -> Result<()> {
        let iface = self.real_interface.clone().unwrap();
        let interface = self.interface_name().to_string();

        let this_peer = self.config.network.peers.get(&self.config.network.this_peer)
            .ok_or_else(|| TunnelError::InvalidConfig("This peer not found".to_string()))?;

        let has_dns = this_peer.dns.enabled && !this_peer.dns.value.is_empty();
        let dns_servers: Vec<String> = if has_dns {
            this_peer.dns.value.split(',').map(|s| s.trim().to_string()).collect()
        } else {
            Vec::new()
        };
        let has_default_route = self.get_allowed_ips()?.iter().any(|ip| ip.ends_with("/0"));
        let mtu_auto = !this_peer.mtu.enabled;

        std::thread::spawn(move || {
            let result = monitor_daemon_worker(
                iface,
                interface,
                has_dns,
                dns_servers,
                has_default_route,
                mtu_auto,
            );

            if let Err(e) = result {
                eprintln!("[!] Monitor daemon error: {}", e);
            }
        });

        eprintln!("[+] Backgrounding route monitor");
        Ok(())
    }

    fn cmd(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            return Err(TunnelError::CommandFailed("Empty command".to_string()));
        }

        eprintln!("[#] {}", args.join(" "));

        let output = Command::new(args[0])
            .args(&args[1..])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stdout);
            return Err(TunnelError::CommandFailed(stderr.to_string()));
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

/// Start a WireGuard tunnel with the given configuration.
/// Returns the real interface name (e.g., "utun3" on macOS or the configured name on Linux).
pub fn start_tunnel(config: Config) -> Result<String> {
    let mut manager = TunnelManager::new(config);
    manager.start_tunnel()
}

/// Stop a WireGuard tunnel with the given configuration.
pub fn stop_tunnel(config: Config) -> Result<()> {
    let mut manager = TunnelManager::new(config);
    manager.stop_tunnel()
}

#[cfg(target_os = "macos")]
fn monitor_daemon_worker(
    real_iface: String,
    _interface: String,
    has_dns: bool,
    dns_servers: Vec<String>,
    has_default_route: bool,
    mtu_auto: bool,
) -> Result<()> {
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

        let iface_check = Command::new("ifconfig")
            .arg(&real_iface)
            .output();

        if iface_check.is_err() || !iface_check.unwrap().status.success() {
            eprintln!("[!] Interface {} no longer exists, stopping monitor", real_iface);
            break;
        }

        if has_default_route {
            if let Err(e) = reapply_endpoint_routes(&real_iface) {
                eprintln!("[!] Failed to reapply endpoint routes: {}", e);
            }
        }

        if mtu_auto {
            if let Err(e) = reapply_mtu(&real_iface) {
                eprintln!("[!] Failed to reapply MTU: {}", e);
            }
        }

        if has_dns && last_dns_update.elapsed() > std::time::Duration::from_secs(2) {
            if let Err(e) = reapply_dns(&dns_servers) {
                eprintln!("[!] Failed to reapply DNS: {}", e);
            }
            last_dns_update = std::time::Instant::now();
        }
    }

    let _ = route_monitor.kill();
    Ok(())
}

#[cfg(target_os = "macos")]
fn reapply_endpoint_routes(real_iface: &str) -> Result<()> {
    let gateway4 = get_default_gateway_helper(false);
    let gateway6 = get_default_gateway_helper(true);

    let output = Command::new("wg")
        .args(&["show", real_iface, "endpoints"])
        .output()?;

    if !output.status.success() {
        return Ok(());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 1 {
            if let Some(endpoint) = extract_ip_from_endpoint(parts[1]) {
                if endpoint.contains(':') {
                    let _ = Command::new("route")
                        .args(&["-q", "-n", "delete", "-inet6", &endpoint])
                        .output();

                    if let Some(gw) = &gateway6 {
                        let _ = Command::new("route")
                            .args(&["-q", "-n", "add", "-inet6", &endpoint, "-gateway", gw])
                            .output();
                    }
                } else {
                    let _ = Command::new("route")
                        .args(&["-q", "-n", "delete", "-inet", &endpoint])
                        .output();

                    if let Some(gw) = &gateway4 {
                        let _ = Command::new("route")
                            .args(&["-q", "-n", "add", "-inet", &endpoint, "-gateway", gw])
                            .output();
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn reapply_mtu(real_iface: &str) -> Result<()> {
    let output = Command::new("ifconfig")
        .arg(real_iface)
        .output()?;

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

    let output = Command::new("netstat")
        .args(&["-nr", "-f", "inet"])
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 5 && parts[0] == "default" {
            let default_iface = parts[5];
            let ifconfig_output = Command::new("ifconfig")
                .arg(default_iface)
                .output()?;

            let ifconfig_str = String::from_utf8_lossy(&ifconfig_output.stdout);
            for line in ifconfig_str.lines() {
                if line.contains("mtu") {
                    if let Some(mtu_str) = line.split("mtu").nth(1) {
                        if let Some(mtu) = mtu_str.trim().split_whitespace().next() {
                            if let Ok(mtu_val) = mtu.parse::<u16>() {
                                let new_mtu = mtu_val.saturating_sub(80);

                                if current_mtu != Some(new_mtu) {
                                    let _ = Command::new("ifconfig")
                                        .args(&[real_iface, "mtu", &new_mtu.to_string()])
                                        .output();
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

#[cfg(target_os = "macos")]
fn reapply_dns(dns_servers: &[String]) -> Result<()> {
    let output = Command::new("networksetup")
        .arg("-listallnetworkservices")
        .output()?;

    let services = String::from_utf8_lossy(&output.stdout);

    for service in services.lines().skip(1) {
        let service = service.trim_start_matches('*').trim();

        let mut args = vec!["-setdnsservers", service];
        let dns_refs: Vec<&str> = dns_servers.iter().map(|s| s.as_str()).collect();
        args.extend(dns_refs);

        let _ = Command::new("networksetup").args(&args).output();
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn get_default_gateway_helper(ipv6: bool) -> Option<String> {
    let family = if ipv6 { "inet6" } else { "inet" };
    let output = Command::new("netstat")
        .args(&["-nr", "-f", family])
        .output()
        .ok()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 1 && parts[0] == "default" && !parts[1].starts_with("link#") {
            return Some(parts[1].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn create_test_config() -> Config {
        let mut peers = HashMap::new();

        let private_key = wg_quickrs_wasm::helpers::wg_generate_key();

        peers.insert(
            "peer1".to_string(),
            wg_quickrs_wasm::types::Peer {
                name: "Test Server".to_string(),
                address: "10.0.0.1/24".to_string(),
                endpoint: wg_quickrs_wasm::types::EnabledValue {
                    enabled: true,
                    value: "0.0.0.0:51820".to_string(),
                },
                kind: "server".to_string(),
                icon: wg_quickrs_wasm::types::EnabledValue::default(),
                dns: wg_quickrs_wasm::types::EnabledValue::default(),
                mtu: wg_quickrs_wasm::types::EnabledValue::default(),
                scripts: wg_quickrs_wasm::types::Scripts::default(),
                private_key,
                created_at: "2025-01-01T00:00:00Z".to_string(),
                updated_at: "2025-01-01T00:00:00Z".to_string(),
            },
        );

        Config {
            agent: wg_quickrs_wasm::types::Agent {
                web: wg_quickrs_wasm::types::AgentWeb {
                    address: "127.0.0.1".to_string(),
                    http: wg_quickrs_wasm::types::AgentWebHttp {
                        enabled: false,
                        port: 8080,
                    },
                    https: wg_quickrs_wasm::types::AgentWebHttps {
                        enabled: false,
                        port: 8443,
                        tls_cert: PathBuf::from("/dev/null"),
                        tls_key: PathBuf::from("/dev/null"),
                    },
                    password: wg_quickrs_wasm::types::Password {
                        enabled: false,
                        hash: "".to_string(),
                    },
                },
                vpn: wg_quickrs_wasm::types::AgentVpn {
                    enabled: true,
                    port: 51820,
                },
                firewall: wg_quickrs_wasm::types::AgentFirewall {
                    enabled: false,
                    utility: PathBuf::from("/usr/bin/ufw"),
                    gateway: "192.168.1.1".to_string(),
                },
            },
            network: wg_quickrs_wasm::types::Network {
                identifier: "wgtest0".to_string(),
                subnet: "10.0.0.0/24".to_string(),
                this_peer: "peer1".to_string(),
                peers,
                connections: HashMap::new(),
                defaults: wg_quickrs_wasm::types::Defaults::default(),
                reservations: HashMap::new(),
                updated_at: "2025-01-01T00:00:00Z".to_string(),
            },
        }
    }

    fn create_config_with_peer() -> Config {
        let mut config = create_test_config();

        let private_key2 = wg_quickrs_wasm::helpers::wg_generate_key();

        config.network.peers.insert(
            "peer2".to_string(),
            wg_quickrs_wasm::types::Peer {
                name: "Test Client".to_string(),
                address: "10.0.0.2/32".to_string(),
                endpoint: wg_quickrs_wasm::types::EnabledValue::default(),
                kind: "client".to_string(),
                icon: wg_quickrs_wasm::types::EnabledValue::default(),
                dns: wg_quickrs_wasm::types::EnabledValue::default(),
                mtu: wg_quickrs_wasm::types::EnabledValue::default(),
                scripts: wg_quickrs_wasm::types::Scripts::default(),
                private_key: private_key2,
                created_at: "2025-01-01T00:00:00Z".to_string(),
                updated_at: "2025-01-01T00:00:00Z".to_string(),
            },
        );

        let connection_id = wg_quickrs_wasm::helpers::get_connection_id("peer1", "peer2");
        config.network.connections.insert(
            connection_id,
            wg_quickrs_wasm::types::Connection {
                enabled: true,
                pre_shared_key: String::new(),
                allowed_ips_a_to_b: "10.0.0.2/32".to_string(),
                allowed_ips_b_to_a: "0.0.0.0/0".to_string(),
                persistent_keepalive: wg_quickrs_wasm::types::EnabledValue::default(),
            },
        );

        config
    }

    fn cleanup_interface(identifier: &str) {
        #[cfg(target_os = "macos")]
        {
            let name_file = format!("/var/run/wireguard/{}.name", identifier);
            if let Ok(iface) = fs::read_to_string(&name_file) {
                let iface = iface.trim();
                let _ = Command::new("ifconfig").args(&[iface, "down"]).output();
                let sock_file = format!("/var/run/wireguard/{}.sock", iface);
                let _ = fs::remove_file(sock_file);
            }
            let _ = fs::remove_file(name_file);
        }

        #[cfg(target_os = "linux")]
        {
            let _ = Command::new("ip")
                .args(&["link", "delete", "dev", identifier])
                .output();
        }
    }

    #[test]
    fn test_extract_ip_from_endpoint() {
        assert_eq!(
            extract_ip_from_endpoint("192.168.1.1:51820"),
            Some("192.168.1.1".to_string())
        );

        assert_eq!(
            extract_ip_from_endpoint("[2001:db8::1]:51820"),
            Some("2001:db8::1".to_string())
        );
    }

    #[test]
    fn test_generate_wg_config() {
        let config = create_config_with_peer();
        let manager = TunnelManager::new(config);

        let wg_config = manager.generate_wg_config().unwrap();

        assert!(wg_config.contains("[Interface]"));
        assert!(wg_config.contains("PrivateKey"));
        assert!(wg_config.contains("ListenPort = 51820"));
        assert!(wg_config.contains("[Peer]"));
        assert!(wg_config.contains("PublicKey"));
        assert!(wg_config.contains("AllowedIPs"));
    }

    #[test]
    #[ignore]
    fn test_interface_lifecycle() {
        let config = create_test_config();
        cleanup_interface(&config.network.identifier);

        let result = start_tunnel(config.clone());
        assert!(result.is_ok(), "Failed to start tunnel: {:?}", result.err());

        std::thread::sleep(std::time::Duration::from_secs(1));

        let result = stop_tunnel(config.clone());
        assert!(result.is_ok(), "Failed to stop tunnel: {:?}", result.err());

        cleanup_interface(&config.network.identifier);
    }

    #[test]
    #[ignore]
    fn test_address_assignment() {
        let config = create_test_config();
        cleanup_interface(&config.network.identifier);

        start_tunnel(config.clone()).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));

        #[cfg(target_os = "linux")]
        let check_cmd = Command::new("ip")
            .args(&["addr", "show", "dev", &config.network.identifier])
            .output()
            .unwrap();

        #[cfg(target_os = "macos")]
        let check_cmd = {
            let name_file = format!("/var/run/wireguard/{}.name", config.network.identifier);
            let iface = fs::read_to_string(&name_file).unwrap();
            Command::new("ifconfig").arg(iface.trim()).output().unwrap()
        };

        let output = String::from_utf8_lossy(&check_cmd.stdout);
        assert!(output.contains("10.0.0.1"), "Address not assigned: {}", output);

        stop_tunnel(config.clone()).unwrap();
        cleanup_interface(&config.network.identifier);
    }

    #[test]
    #[ignore]
    fn test_peer_configuration() {
        let config = create_config_with_peer();
        cleanup_interface(&config.network.identifier);

        start_tunnel(config.clone()).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));

        #[cfg(target_os = "linux")]
        let real_iface = &config.network.identifier;

        #[cfg(target_os = "macos")]
        let real_iface = {
            let name_file = format!("/var/run/wireguard/{}.name", config.network.identifier);
            fs::read_to_string(&name_file).unwrap().trim().to_string()
        };

        let output = Command::new("wg")
            .args(&["show", &real_iface])
            .output()
            .unwrap();

        let wg_output = String::from_utf8_lossy(&output.stdout);
        assert!(wg_output.contains("peer:"), "No peer configured: {}", wg_output);

        stop_tunnel(config.clone()).unwrap();
        cleanup_interface(&config.network.identifier);
    }

    #[test]
    #[ignore]
    fn test_hooks_execution() {
        let mut config = create_test_config();
        let test_file = "/tmp/wgtest_hook";
        let _ = fs::remove_file(test_file);

        if let Some(peer) = config.network.peers.get_mut("peer1") {
            peer.scripts.post_up = vec![wg_quickrs_wasm::types::EnabledValue {
                enabled: true,
                value: format!("touch {}", test_file),
            }];
            peer.scripts.pre_down = vec![wg_quickrs_wasm::types::EnabledValue {
                enabled: true,
                value: format!("rm -f {}", test_file),
            }];
        }

        cleanup_interface(&config.network.identifier);
        start_tunnel(config.clone()).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));

        assert!(PathBuf::from(test_file).exists(), "PostUp hook not executed");

        stop_tunnel(config.clone()).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));

        assert!(!PathBuf::from(test_file).exists(), "PreDown hook not executed");
        cleanup_interface(&config.network.identifier);
    }
}