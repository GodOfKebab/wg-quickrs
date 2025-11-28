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
    #[command(about = "Set path to WireGuard binary")]
    Wg {
        #[arg(help = "Path to WireGuard binary")]
        value: PathBuf,
    },
    #[command(about = "Set WireGuard userspace configuration")]
    WgUserspace {
        #[command(subcommand)]
        target: SetAgentVpnWgUserspaceCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum SetAgentVpnWgUserspaceCommands {
    #[command(about = "Set path to WireGuard userspace binary")]
    Binary {
        #[arg(help = "Path to WireGuard userspace binary")]
        value: PathBuf,
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
    #[command(about = "Set AmneziaWG network parameters")]
    AmneziaParameters {
        #[command(subcommand)]
        target: SetNetworkAmneziaParametersCommands,
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
    #[command(about = "Set peer AmneziaWG parameters")]
    AmneziaParameters {
        #[command(subcommand)]
        target: SetPeerAmneziaParametersCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum SetPeerAmneziaParametersCommands {
    #[command(about = "Set Jc parameter (junk packet count)")]
    Jc {
        #[arg(help = "Junk packet count")]
        value: i16,
    },
    #[command(about = "Set Jmin parameter (minimum junk packet size)")]
    Jmin {
        #[arg(help = "Minimum junk packet size")]
        value: u16,
    },
    #[command(about = "Set Jmax parameter (maximum junk packet size)")]
    Jmax {
        #[arg(help = "Maximum junk packet size")]
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
    #[command(about = "Set default peer AmneziaWG parameters")]
    AmneziaParameters {
        #[command(subcommand)]
        target: SetPeerAmneziaParametersCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum SetNetworkAmneziaParametersCommands {
    #[command(about = "Set S1 parameter (init packet junk size)")]
    S1 {
        #[arg(help = "S1 value")]
        value: u16,
    },
    #[command(about = "Set S2 parameter (response packet junk size)")]
    S2 {
        #[arg(help = "S2 value")]
        value: u16,
    },
    #[command(about = "Set H1 parameter (init packet magic header)")]
    H1 {
        #[arg(help = "H1 value")]
        value: u32,
    },
    #[command(about = "Set H2 parameter (response packet magic header)")]
    H2 {
        #[arg(help = "H2 value")]
        value: u32,
    },
    #[command(about = "Set H3 parameter (underload packet magic header)")]
    H3 {
        #[arg(help = "H3 value")]
        value: u32,
    },
    #[command(about = "Set H4 parameter (transport packet magic header)")]
    H4 {
        #[arg(help = "H4 value")]
        value: u32,
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
