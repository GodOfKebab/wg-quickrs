use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// The main CLI struct
#[derive(Parser, Debug)]
#[command(
    // version will be set by the implementation
    name = "wg-quickrs",
    about = "A tool to manage the peer and network configuration of the \
             WireGuard-based overlay network over the web console",
)]
pub struct Cli {
    #[arg(short, long, help = "Increase verbosity level from Info to Debug")]
    pub verbose: bool,
    #[cfg(target_os = "macos")]
    #[arg(long, default_value = "/opt/homebrew/etc/wg-quickrs/")]
    pub wg_quickrs_config_folder: PathBuf,
    #[cfg(target_os = "linux")]
    #[arg(long, default_value = "/etc/wg-quickrs/")]
    pub wg_quickrs_config_folder: PathBuf,
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    #[arg(long)]
    pub wg_quickrs_config_folder: PathBuf,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(
        about = "Initialize the wg-quickrs agent.\nConfiguration options can be filled either by prompts on screen (when no argument is provided) or specified as arguments to this command"
    )]
    Init(Box<InitOptions>),
    #[command(about = "Configure and run the wg-quickrs agent")]
    Agent {
        #[cfg(target_os = "macos")]
        #[arg(long, default_value = "/opt/homebrew/etc/wireguard/")]
        wireguard_config_folder: PathBuf,
        #[cfg(target_os = "linux")]
        #[arg(long, default_value = "/etc/wireguard/")]
        wireguard_config_folder: PathBuf,
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        #[arg(long)]
        wireguard_config_folder: PathBuf,
        #[command(subcommand)]
        commands: AgentCommands,
    },
}

#[derive(Debug, Args)]
pub struct InitOptions {
    #[arg(long, default_value = None, long_help = "Set VPN network identifier", value_name = "wg-quickrs"
    )]
    pub network_identifier: Option<String>,

    #[arg(long, default_value = None, long_help = "Set VPN network CIDR subnet", value_name = "10.0.34.0/24"
    )]
    pub network_subnet: Option<String>,

    #[arg(long, default_value = None, long_help = "Set agent web server bind IPv4 address"
    )]
    pub agent_web_address: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable HTTP on web server")]
    pub agent_web_http_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set web server HTTP port", value_name = "80"
    )]
    pub agent_web_http_port: Option<u16>,

    #[arg(long, default_value = None, long_help = "Enable HTTPS on web server")]
    pub agent_web_https_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set web server HTTPS port", value_name = "443"
    )]
    pub agent_web_https_port: Option<u16>,

    #[arg(long, default_value = None, long_help = "Set path (relative to the wg-quickrs config folder) to TLS certificate file for HTTPS", value_name = "certs/servers/localhost/cert.pem"
    )]
    pub agent_web_https_tls_cert: Option<PathBuf>,

    #[arg(long, default_value = None, long_help = "Set path (relative to the wg-quickrs config folder) to TLS private key file for HTTPS", value_name = "certs/servers/localhost/key.pem"
    )]
    pub agent_web_https_tls_key: Option<PathBuf>,

    #[arg(long, default_value = None, long_help = "Enable password authentication for web server")]
    pub agent_web_password_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set password for web server access")]
    pub agent_web_password: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable VPN server"
    )]
    pub agent_vpn_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set VPN server listening port", value_name = "51820"
    )]
    pub agent_vpn_port: Option<u16>,

    #[arg(long, default_value = None, long_help = "Set gateway (outbound interface) for VPN packet forwarding", value_name = "eth0"
    )]
    pub agent_vpn_gateway: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable running firewall commands for setting up NAT and input rules"
    )]
    pub agent_firewall_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set the utility used to configure firewall NAT and input rules", value_name = "iptables"
    )]
    pub agent_firewall_utility: Option<PathBuf>,

    #[arg(long, default_value = None, long_help = "Set agent peer name", value_name = "wg-quickrs-host"
    )]
    pub agent_peer_name: Option<String>,

    #[arg(long, default_value = None, long_help = "Set internal IPv4 address for agent in VPN network", value_name = "10.0.34.1"
    )]
    pub agent_peer_vpn_internal_address: Option<String>,

    #[arg(long, default_value = None, long_help = "Set publicly accessible endpoint(IP/FQDN:PORT) for VPN endpoint"
    )]
    pub agent_peer_vpn_endpoint: Option<String>,

    #[arg(long, default_value = None, long_help = "Set peer kind for agent"
    )]
    pub agent_peer_kind: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable peer icon for agent"
    )]
    pub agent_peer_icon_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set peer icon for agent"
    )]
    pub agent_peer_icon_src: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable DNS configuration for agent")]
    pub agent_peer_dns_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set DNS server for agent", value_name = "1.1.1.1"
    )]
    pub agent_peer_dns_server: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable MTU configuration for agent")]
    pub agent_peer_mtu_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set MTU value for agent", value_name = "1420")]
    pub agent_peer_mtu_value: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable PreUp script for agent")]
    pub agent_peer_script_pre_up_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set PreUp script line for agent"
    )]
    pub agent_peer_script_pre_up_line: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable PostUp script for agent")]
    pub agent_peer_script_post_up_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set PostUp script line for agent"
    )]
    pub agent_peer_script_post_up_line: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable PreDown script for agent"
    )]
    pub agent_peer_script_pre_down_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set PreDown script line for agent"
    )]
    pub agent_peer_script_pre_down_line: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable PostDown script for agent"
    )]
    pub agent_peer_script_post_down_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set PostDown script line for agent"
    )]
    pub agent_peer_script_post_down_line: Option<String>,

    // default settings for new peers/connections
    #[arg(long, default_value = None, long_help = "Set peer kind for new peers by default"
    )]
    pub default_peer_kind: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable peer icon for new peers by default"
    )]
    pub default_peer_icon_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set peer icon for new peers by default"
    )]
    pub default_peer_icon_src: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable DNS for new peers by default")]
    pub default_peer_dns_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set default DNS server for new peers", value_name = "1.1.1.1"
    )]
    pub default_peer_dns_server: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable MTU for new peers by default")]
    pub default_peer_mtu_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set default MTU value for new peers", value_name = "1420"
    )]
    pub default_peer_mtu_value: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable PreUp script for new peers by default"
    )]
    pub default_peer_script_pre_up_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set default PreUp script line for new peers"
    )]
    pub default_peer_script_pre_up_line: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable PostUp script for new peers by default"
    )]
    pub default_peer_script_post_up_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set default PostUp script line for new peers"
    )]
    pub default_peer_script_post_up_line: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable PreDown script for new peers by default"
    )]
    pub default_peer_script_pre_down_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set default PreDown script line for new peers"
    )]
    pub default_peer_script_pre_down_line: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable PostDown script for new peers by default"
    )]
    pub default_peer_script_post_down_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set default PostDown script line for new peers"
    )]
    pub default_peer_script_post_down_line: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable PersistentKeepalive for new connections by default"
    )]
    pub default_connection_persistent_keepalive_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set default PersistentKeepalive period in seconds", value_name = "25"
    )]
    pub default_connection_persistent_keepalive_period: Option<String>,

    #[arg(long, default_value = None, long_help = "Disable interactive setup prompts")]
    pub no_prompt: Option<bool>,
}

#[derive(Subcommand, Debug)]
pub enum AgentCommands {
    #[command(about = "Runs the wg-quickrs")]
    Run,
    // setting: address
    #[command(about = "Set agent web server bind IPv4 address")]
    SetWebAddress(AddressArg),
    // settings: http
    #[command(about = "Enable HTTP on web server")]
    EnableWebHttp,
    #[command(about = "Disable HTTP on web server")]
    DisableWebHttp,
    #[command(about = "Set web server HTTP port")]
    SetHttpWebPort(PortArg),
    // settings: https
    #[command(about = "Enable HTTPS on web server")]
    EnableWebHttps,
    #[command(about = "Disable HTTPS on web server")]
    DisableWebHttps,
    #[command(about = "Set port for the HTTPS web server")]
    SetWebHttpsPort(PortArg),
    #[command(
        about = "Set path (relative to the wg-quickrs config folder) to TLS certificate file for HTTPS"
    )]
    SetWebHttpsTlsCert(PathArg),
    #[command(
        about = "Set path (relative to the wg-quickrs config folder) to TLS private key file for HTTPS"
    )]
    SetWebHttpsTlsKey(PathArg),
    // settings: password
    #[command(about = "Enable password authentication for web server")]
    EnableWebPassword,
    #[command(about = "Disable password authentication for web server")]
    DisableWebPassword,
    #[command(about = "Set password for web server access")]
    ResetWebPassword(ResetWebPasswordOptions),
    // setting: VPN
    #[command(about = "Enable VPN server")]
    EnableVpn,
    #[command(about = "Disable VPN server")]
    DisableVpn,
    #[command(about = "Set VPN server listening port")]
    SetVpnPort(PortArg),
    // setting: Firewall
    #[command(about = "Enable running firewall commands for setting up NAT and input rules")]
    EnableFirewall,
    #[command(about = "Disable running firewall commands for setting up NAT and input rules")]
    DisableFirewall,
    #[command(
        about = "Set the utility used to configure firewall NAT and input rules (e.g. iptables, pfctl, etc.)"
    )]
    SetFirewallUtility(UtilityArg),
}

#[derive(Debug, Args)]
pub struct AddressArg {
    #[arg(help = "IPv4 address")]
    pub address: String,
}

#[derive(Debug, Args)]
pub struct PortArg {
    #[arg(help = "Port number(0-65535)")]
    pub port: u16,
}

#[derive(Debug, Args)]
pub struct PathArg {
    #[arg(help = "File path")]
    pub path: PathBuf,
}

#[derive(Debug, Args)]
pub struct UtilityArg {
    #[arg(help = "Utility binary path or name")]
    pub utility: PathBuf,
}

#[derive(Args, Debug)]
pub struct ResetWebPasswordOptions {
    #[arg(long, default_value = None, help = "The use of this option is HIGHLY DISCOURAGED because the plaintext password might show up in the shell history! THIS IS HIGHLY INSECURE! Please set the password without the --password flag, and the script will prompt for the password."
    )]
    pub password: Option<String>,
}
