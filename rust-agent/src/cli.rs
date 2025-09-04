use crate::macros::*;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    version = full_version!(),
    about = "A tool to manage the peer and network configuration of the \
             WireGuard-based overlay network over the web console",
)]
pub(crate) struct Cli {
    #[arg(short, long, help = "Increase verbosity level from Info to Debug")]
    pub(crate) verbose: bool,
    #[arg(long, default_value = "~/.wg-rusteze")]
    pub(crate) wg_rusteze_config_folder: PathBuf,
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    #[command(
        about = "Initialize the wg-rusteze rust-agent.\nConfiguration options can be filled either by prompts on screen (when no argument is provided) or specified as arguments to this command"
    )]
    Init(Box<InitOptions>),
    #[command(about = "Configure and run the wg-rusteze rust-agent")]
    Agent {
        #[cfg(target_os = "macos")]
        #[arg(long, default_value = "/opt/homebrew/etc/wireguard/")]
        wireguard_config_folder: PathBuf,
        #[cfg(target_os = "linux")]
        #[arg(long, default_value = "/etc/wireguard/")]
        wireguard_config_folder: PathBuf,
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        #[arg(long, default_value = "/tmp/wireguard/")]
        wireguard_config_folder: PathBuf,
        #[command(subcommand)]
        commands: AgentCommands,
    },
}

#[derive(Debug, Args)]
pub struct InitOptions {
    #[arg(long, default_value = None, help = "VPN network's identifier (e.g. wg-rusteze)")]
    pub network_identifier: Option<String>,

    #[arg(long, default_value = None, help = "VPN network's CIDR subnet mask (e.g. 10.0.34.0/24)")]
    pub network_subnet: Option<String>,

    #[arg(long, default_value = None, help = "Agent's peer name (e.g. wg-rusteze-host)")]
    pub agent_peer_name: Option<String>,

    #[arg(long, default_value = None, help = "Agent's local IPv4 address for the web server to bind"
    )]
    pub agent_local_address: Option<String>,

    #[arg(long, default_value = None, help = "Enable/Disable HTTP for the web server")]
    pub agent_local_enable_web_http: Option<bool>,

    #[arg(long, default_value = None, help = "Agent's local web port for the web server (HTTP) to bind (e.g. 80)"
    )]
    pub agent_local_web_http_port: Option<u16>,

    #[arg(long, default_value = None, help = "Enable/Disable HTTPS for the web server")]
    pub agent_local_enable_web_https: Option<bool>,

    #[arg(long, default_value = None, help = "Agent's local web port for the web server (HTTPS) to bind (e.g. 443)"
    )]
    pub agent_local_web_https_port: Option<u16>,

    #[arg(long, default_value = None, help = "TLS certificate file path for HTTPS")]
    pub agent_local_web_https_tls_cert: Option<PathBuf>,

    #[arg(long, default_value = None, help = "TLS signing key file path for HTTPS")]
    pub agent_local_web_https_tls_key: Option<PathBuf>,

    #[arg(long, default_value = None, help = "Enable/Disable VPN server"
    )]
    pub agent_local_enable_vpn: Option<bool>,

    #[arg(long, default_value = None, help = "Agent's local VPN port for the VPN server listen (e.g. 51820)"
    )]
    pub agent_local_vpn_port: Option<u16>,

    #[arg(long, default_value = None, help = "Agent's local interface for the VPN server's packet forwarding setup (e.g. eth0)"
    )]
    pub agent_local_vpn_interface: Option<String>,

    #[arg(long, default_value = None, help = "Agent's publicly accessible IPv4 address to be used in the VPN endpoint advertisement"
    )]
    pub agent_public_address: Option<String>,

    #[arg(long, default_value = None, help = "Agent's publicly accessible port to be used in the VPN endpoint advertisement (e.g. 51820)"
    )]
    pub agent_public_vpn_port: Option<u16>,

    #[arg(long, default_value = None, help = "Agent's internal IPv4 address for VPN network (e.g. 10.0.34.1)"
    )]
    pub agent_internal_vpn_address: Option<String>,

    #[arg(long, default_value = None, help = "Enable/Disable password for the web server")]
    pub agent_enable_web_password: Option<bool>,

    #[arg(long, default_value = None, help = "Agent's password for the web server")]
    pub agent_web_password: Option<String>,

    #[arg(long, default_value = None, help = "Enable/Disable DNS field for the agent")]
    pub agent_enable_dns: Option<bool>,

    #[arg(long, default_value = None, help = "Agent's DNS field setting as an IPv4 address (e.g. 1.1.1.1)"
    )]
    pub agent_dns_server: Option<String>,

    #[arg(long, default_value = None, help = "Enable/Disable MTU field for the agent")]
    pub agent_enable_mtu: Option<bool>,

    #[arg(long, default_value = None, help = "Agent's MTU field setting as a value (e.g. 1420)")]
    pub agent_mtu_value: Option<String>,

    #[arg(long, default_value = None, help = "Enable/Disable PreUp scripting field for the agent")]
    pub agent_enable_script_pre_up: Option<bool>,

    #[arg(long, default_value = None, help = "Agent's PreUp scripting field setting as a line of script (e.g. sysctl -w net.ipv4.ip_forward=1)"
    )]
    pub agent_script_pre_up_line: Option<String>,

    #[arg(long, default_value = None, help = "Enable/Disable PostUp scripting field for the agent")]
    pub agent_enable_script_post_up: Option<bool>,

    #[arg(long, default_value = None, help = "Agent's PostUp scripting field setting as a line of script (e.g. echo 'Network is up')"
    )]
    pub agent_script_post_up_line: Option<String>,

    #[arg(long, default_value = None, help = "Enable/Disable PreDown scripting field for the agent"
    )]
    pub agent_enable_script_pre_down: Option<bool>,

    #[arg(long, default_value = None, help = "Agent's PreDown scripting field setting as a line of script (e.g. echo 'Network is going down')"
    )]
    pub agent_script_pre_down_line: Option<String>,

    #[arg(long, default_value = None, help = "Enable/Disable PostDown scripting field for the agent"
    )]
    pub agent_enable_script_post_down: Option<bool>,

    #[arg(long, default_value = None, help = "Agent's PostDown scripting field setting as a line of script (e.g. sysctl -w net.ipv4.ip_forward=0)"
    )]
    pub agent_script_post_down_line: Option<String>,

    // default settings for new peers/connections
    #[arg(long, default_value = None, help = "Enable/Disable DNS field for new peers by default")]
    pub default_enable_dns: Option<bool>,

    #[arg(long, default_value = None, help = "Default DNS field setting as an IPv4 address for the new peer (e.g. 1.1.1.1)"
    )]
    pub default_dns_server: Option<String>,

    #[arg(long, default_value = None, help = "Enable/Disable MTU field for new peers by default")]
    pub default_enable_mtu: Option<bool>,

    #[arg(long, default_value = None, help = "Default MTU field setting as a value for the new peer (e.g. 1420)"
    )]
    pub default_mtu_value: Option<String>,

    #[arg(long, default_value = None, help = "Enable/Disable PreUp scripting field for new peers by default"
    )]
    pub default_enable_script_pre_up: Option<bool>,

    #[arg(long, default_value = None, help = "New Peer's default PreUp scripting field setting as a line of script (e.g. sysctl -w net.ipv4.ip_forward=1)"
    )]
    pub default_script_pre_up_line: Option<String>,

    #[arg(long, default_value = None, help = "Enable/Disable PostUp scripting field for new peers by default"
    )]
    pub default_enable_script_post_up: Option<bool>,

    #[arg(long, default_value = None, help = "New Peer's default PostUp scripting field setting as a line of script (e.g. echo 'Network is up')"
    )]
    pub default_script_post_up_line: Option<String>,

    #[arg(long, default_value = None, help = "Enable/Disable PreDown scripting field for new peers by default"
    )]
    pub default_enable_script_pre_down: Option<bool>,

    #[arg(long, default_value = None, help = "New Peer's default PreDown scripting field setting as a line of script (e.g. echo 'Network is going down')"
    )]
    pub default_script_pre_down_line: Option<String>,

    #[arg(long, default_value = None, help = "Enable/Disable PostDown scripting field for the agent"
    )]
    pub default_enable_script_post_down: Option<bool>,

    #[arg(long, default_value = None, help = "Agent's PostDown scripting field setting as a line of script (e.g. sysctl -w net.ipv4.ip_forward=0)"
    )]
    pub default_script_post_down_line: Option<String>,

    #[arg(long, default_value = None, help = "Enable/Disable PersistentKeepalive field for new connections by default"
    )]
    pub default_enable_persistent_keepalive: Option<bool>,

    #[arg(long, default_value = None, help = "Default PersistentKeepalive period in seconds (e.g. 25)"
    )]
    pub default_persistent_keepalive_period: Option<String>,

    #[arg(long, default_value = None, help = "Disable setting up the agent with prompts")]
    pub no_prompt: Option<bool>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum AgentCommands {
    #[command(about = "Runs the rust-agent")]
    Run,
    // setting: address
    #[command(
        about = "Set the local IPv4 address for the web server to bind and vpn server to listen"
    )]
    SetAddress(AddressArg),
    // settings: http
    #[command(about = "Enable the HTTP web server")]
    EnableWebHttp,
    #[command(about = "Disable the HTTP web server")]
    DisableWebHttp,
    #[command(about = "Set port for the HTTP web server")]
    SetHttpWebPort(PortArg),
    // settings: https
    #[command(about = "Enable the HTTPS web server")]
    EnableWebHttps,
    #[command(about = "Disable the HTTPS web server")]
    DisableWebHttps,
    #[command(about = "Set port for the HTTPS web server")]
    SetWebHttpsPort(PortArg),
    #[command(
        about = "Set TLS certificate file path (relative to the wg-rusteze home directory) for HTTPS web server"
    )]
    SetWebHttpsTlsCert(PathArg),
    #[command(
        about = "Set TLS signing key file path (relative to the wg-rusteze home directory) for HTTPS web server"
    )]
    SetWebHttpsTlsKey(PathArg),
    // setting: VPN
    #[command(about = "Enable the VPN server")]
    EnableVpn,
    #[command(about = "Disable the VPN server")]
    DisableVpn,
    #[command(about = "Set port for the VPN server")]
    SetVpnPort(PortArg),
    // settings: password
    #[command(about = "Enable the web password")]
    EnableWebPassword,
    #[command(about = "Disable the web password")]
    DisableWebPassword,
    #[command(about = "Reset the web password")]
    ResetWebPassword(ResetWebPasswordOptions),
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

#[derive(Args, Debug)]
pub(crate) struct ResetWebPasswordOptions {
    #[arg(long, default_value = None, help = "The use of this option is HIGHLY DISCOURAGED because the plaintext password might show up in the shell history! THIS IS HIGHLY INSECURE! Please set the password without the --password flag, and the script will prompt for the password."
    )]
    pub password: Option<String>,
}
