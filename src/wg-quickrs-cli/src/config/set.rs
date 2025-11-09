use std::net::Ipv4Addr;
use std::path::PathBuf;
use clap::Subcommand;
use uuid::Uuid;

#[derive(Subcommand, Debug)]
pub enum SetCommands {
    #[command(about = "Set agent configuration values")]
    Agent {
        #[command(subcommand)]
        target: SetAgentCommands,
    },
    #[command(about = "Set network configuration values")]
    Network {
        #[command(subcommand)]
        target: SetNetworkCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum SetAgentCommands {
    #[command(about = "Set web server configuration")]
    Web {
        #[command(subcommand)]
        target: SetAgentWebCommands,
    },
    #[command(about = "Set VPN configuration")]
    Vpn {
        #[command(subcommand)]
        target: SetAgentVpnCommands,
    },
    #[command(about = "Set firewall configuration")]
    Firewall {
        #[command(subcommand)]
        target: SetAgentFirewallCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum SetAgentWebCommands {
    #[command(about = "Set agent web server bind IPv4 address")]
    Address {
        #[arg(help = "IPv4 address")]
        value: Ipv4Addr,
    },
    #[command(about = "Set HTTP configuration")]
    Http {
        #[command(subcommand)]
        target: SetAgentWebHttpCommands,
    },
    #[command(about = "Set HTTPS configuration")]
    Https {
        #[command(subcommand)]
        target: SetAgentWebHttpsCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum SetAgentWebHttpCommands {
    #[command(about = "Set web server HTTP port")]
    Port {
        #[arg(help = "Port number (0-65535)")]
        value: u16,
    },
}

#[derive(Subcommand, Debug)]
pub enum SetAgentWebHttpsCommands {
    #[command(about = "Set web server HTTPS port")]
    Port {
        #[arg(help = "Port number (0-65535)")]
        value: u16,
    },
    #[command(about = "Set path (relative to the wg-quickrs config folder) to TLS certificate file for HTTPS")]
    TlsCert {
        #[arg(help = "File path")]
        value: PathBuf,
    },
    #[command(about = "Set path (relative to the wg-quickrs config folder) to TLS private key file for HTTPS")]
    TlsKey {
        #[arg(help = "File path")]
        value: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum SetAgentVpnCommands {
    #[command(about = "Set VPN server listening port")]
    Port {
        #[arg(help = "Port number (0-65535)")]
        value: u16,
    },
}

#[derive(Subcommand, Debug)]
pub enum SetAgentFirewallCommands {
    #[command(about = "Set the utility used to configure firewall NAT and input rules (e.g. iptables, pfctl, etc.)")]
    Utility {
        #[arg(help = "Utility binary path or name")]
        value: PathBuf,
    },
    #[command(about = "Set the gateway used to configure firewall NAT and input rules (e.g. en0, eth0, etc.)")]
    Gateway {
        #[arg(help = "Internet interface name")]
        value: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum SetNetworkCommands {
    #[command(about = "Set network name")]
    Name {
        #[arg(help = "New network name")]
        name: String,
    },
    #[command(about = "Set network subnet")]
    Subnet {
        #[arg(help = "New subnet (e.g., 10.0.0.0/24)")]
        subnet: String,
    },
    #[command(about = "Set peer configuration")]
    Peer {
        #[arg(help = "Peer UUID")]
        id: Uuid,
        #[command(subcommand)]
        target: SetPeerCommands,
    },
    #[command(about = "Set connection configuration")]
    Connection {
        #[arg(help = "Connection ID (format: uuid*uuid)")]
        id: String,
        #[command(subcommand)]
        target: SetConnectionCommands,
    },
    #[command(about = "Set default configuration")]
    Defaults {
        #[command(subcommand)]
        target: SetDefaultsCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum SetPeerCommands {
    #[command(about = "Set peer name")]
    Name {
        #[arg(help = "New peer name")]
        name: String,
    },
    #[command(about = "Set peer address")]
    Address {
        #[arg(help = "New IPv4 address")]
        address: Ipv4Addr,
    },
    #[command(about = "Set peer endpoint address")]
    Endpoint {
        #[arg(help = "Endpoint address (hostname:port or ipv4:port)")]
        endpoint: String,
    },
    #[command(about = "Set peer kind")]
    Kind {
        #[arg(help = "Peer kind (e.g., laptop, server, phone)")]
        kind: String,
    },
    #[command(about = "Set peer icon source")]
    Icon {
        #[arg(help = "Icon source (URL or path)")]
        src: String,
    },
    #[command(about = "Set peer DNS addresses")]
    Dns {
        #[arg(help = "Comma-separated list of IPv4 addresses (e.g., 8.8.8.8,8.8.4.4)")]
        addresses: String,
    },
    #[command(about = "Set peer MTU value")]
    Mtu {
        #[arg(help = "MTU value")]
        value: u16,
    },
}

#[derive(Subcommand, Debug)]
pub enum SetConnectionCommands {
    #[command(about = "Set allowed IPs from peer A to peer B")]
    AllowedIpsAToB {
        #[arg(help = "Comma-separated list of CIDR blocks (e.g., 0.0.0.0/0,10.0.0.0/8)")]
        ips: String,
    },
    #[command(about = "Set allowed IPs from peer B to peer A")]
    AllowedIpsBToA {
        #[arg(help = "Comma-separated list of CIDR blocks")]
        ips: String,
    },
    #[command(about = "Set persistent keepalive period")]
    PersistentKeepalive {
        #[arg(help = "Keepalive period in seconds")]
        period: u16,
    },
}

#[derive(Subcommand, Debug)]
pub enum SetDefaultsCommands {
    #[command(about = "Set default peer configuration")]
    Peer {
        #[command(subcommand)]
        target: SetDefaultsPeerCommands,
    },
    #[command(about = "Set default connection configuration")]
    Connection {
        #[command(subcommand)]
        target: SetDefaultsConnectionCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum SetDefaultsPeerCommands {
    #[command(about = "Set default peer kind")]
    Kind {
        #[arg(help = "Peer kind (e.g., laptop, server, phone)")]
        kind: String,
    },
    #[command(about = "Set default peer endpoint address")]
    Endpoint {
        #[arg(help = "Endpoint address (hostname:port or ipv4:port)")]
        endpoint: String,
    },
    #[command(about = "Set default peer icon source")]
    Icon {
        #[arg(help = "Icon source (URL or path)")]
        src: String,
    },
    #[command(about = "Set default peer DNS addresses")]
    Dns {
        #[arg(help = "Comma-separated list of IPv4 addresses (e.g., 8.8.8.8,8.8.4.4)")]
        addresses: String,
    },
    #[command(about = "Set default peer MTU value")]
    Mtu {
        #[arg(help = "MTU value")]
        value: u16,
    },
}

#[derive(Subcommand, Debug)]
pub enum SetDefaultsConnectionCommands {
    #[command(about = "Set default connection persistent keepalive period")]
    PersistentKeepalive {
        #[arg(help = "Keepalive period in seconds")]
        period: u16,
    },
}
