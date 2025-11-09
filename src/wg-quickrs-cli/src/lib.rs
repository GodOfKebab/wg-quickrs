use std::net::Ipv4Addr;
use ipnet::Ipv4Net;
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
    #[arg(long, default_value = "/etc/wg-quickrs/")]
    pub wg_quickrs_config_folder: PathBuf,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Run agent commands")]
    Agent {
        #[command(subcommand)]
        target: AgentCommands,
    },
    #[command(about = "Edit agent configuration options")]
    Config {
        #[command(subcommand)]
        target: ConfigCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum AgentCommands {
    #[command(
        about = "Initialize the wg-quickrs agent.\nConfiguration options can be filled either by prompts on screen (when no argument is provided) or specified as arguments to this command"
    )]
    Init(Box<InitOptions>),
    #[command(about = "Run the wg-quickrs agent")]
    Run,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    #[command(about = "Enable a configuration option")]
    Enable {
        #[command(subcommand)]
        target: EnableCommands,
    },
    #[command(about = "Disable a configuration option")]
    Disable {
        #[command(subcommand)]
        target: DisableCommands,
    },
    #[command(about = "Set a configuration value")]
    Set {
        #[command(subcommand)]
        target: SetCommands,
    },
    #[command(about = "Reset a configuration option")]
    Reset {
        #[command(subcommand)]
        target: ResetCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum EnableCommands {
    #[command(about = "Enable agent configuration options")]
    Agent {
        #[command(subcommand)]
        target: EnableAgentCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum EnableAgentCommands {
    #[command(about = "Enable web server options")]
    Web {
        #[command(subcommand)]
        target: EnableAgentWebCommands,
    },
    #[command(about = "Enable VPN server")]
    Vpn,
    #[command(about = "Enable firewall configuration")]
    Firewall,
}

#[derive(Subcommand, Debug)]
pub enum EnableAgentWebCommands {
    #[command(about = "Enable HTTP on web server")]
    Http,
    #[command(about = "Enable HTTPS on web server")]
    Https,
    #[command(about = "Enable password authentication for web server")]
    Password,
}

#[derive(Subcommand, Debug)]
pub enum DisableCommands {
    #[command(about = "Disable agent configuration options")]
    Agent {
        #[command(subcommand)]
        target: DisableAgentCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum DisableAgentCommands {
    #[command(about = "Disable web server options")]
    Web {
        #[command(subcommand)]
        target: DisableAgentWebCommands,
    },
    #[command(about = "Disable VPN server")]
    Vpn,
    #[command(about = "Disable firewall configuration")]
    Firewall,
}

#[derive(Subcommand, Debug)]
pub enum DisableAgentWebCommands {
    #[command(about = "Disable HTTP on web server")]
    Http,
    #[command(about = "Disable HTTPS on web server")]
    Https,
    #[command(about = "Disable password authentication for web server")]
    Password,
}

#[derive(Subcommand, Debug)]
pub enum SetCommands {
    #[command(about = "Set agent configuration values")]
    Agent {
        #[command(subcommand)]
        target: SetAgentCommands,
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
pub enum ResetCommands {
    #[command(about = "Reset agent configuration options")]
    Agent {
        #[command(subcommand)]
        target: ResetAgentCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ResetAgentCommands {
    #[command(about = "Reset web server configuration")]
    Web {
        #[command(subcommand)]
        target: ResetAgentWebCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ResetAgentWebCommands {
    #[command(about = "Reset password for web server access")]
    Password {
        #[arg(long, help = "The use of this option is HIGHLY DISCOURAGED because the plaintext password might show up in the shell history! THIS IS HIGHLY INSECURE! Please set the password without the --password flag, and the script will prompt for the password.")]
        password: Option<String>,
    },
}

#[derive(Debug, Args)]
pub struct InitOptions {
    #[arg(long, default_value = None, long_help = "Set VPN network name", value_name = "wg-quickrs-home"
    )]
    pub network_name: Option<String>,

    #[arg(long, default_value = None, long_help = "Set VPN network CIDR subnet", value_name = "10.0.34.0/24"
    )]
    pub network_subnet: Option<Ipv4Net>,

    #[arg(long, default_value = None, long_help = "Set agent web server bind IPv4 address"
    )]
    pub agent_web_address: Option<Ipv4Addr>,

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

    #[arg(long, default_value = None, long_help = "Enable running firewall commands for setting up NAT and input rules"
    )]
    pub agent_firewall_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set the utility used to configure firewall NAT and input rules", value_name = "iptables"
    )]
    pub agent_firewall_utility: Option<PathBuf>,

    #[arg(long, default_value = None, long_help = "Set gateway (outbound interface) for VPN packet forwarding", value_name = "eth0"
    )]
    pub agent_firewall_gateway: Option<String>,

    #[arg(long, default_value = None, long_help = "Set agent peer name", value_name = "wg-quickrs-host"
    )]
    pub agent_peer_name: Option<String>,

    #[arg(long, default_value = None, long_help = "Set internal IPv4 address for agent in VPN network", value_name = "10.0.34.1"
    )]
    pub agent_peer_vpn_internal_address: Option<Ipv4Addr>,

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

    #[arg(long, default_value = None, long_help = "Set DNS addresses for agent", value_name = "1.1.1.1"
    )]
    pub agent_peer_dns_addresses: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable MTU configuration for agent")]
    pub agent_peer_mtu_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set MTU value for agent", value_name = "1420")]
    pub agent_peer_mtu_value: Option<u16>,

    #[arg(long, default_value = None, long_help = "Enable PreUp script for agent")]
    pub agent_peer_script_pre_up_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set PreUp script line(s) for agent. Can be specified multiple times for multiple script lines."
    )]
    pub agent_peer_script_pre_up_line: Vec<String>,

    #[arg(long, default_value = None, long_help = "Enable PostUp script for agent")]
    pub agent_peer_script_post_up_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set PostUp script line(s) for agent. Can be specified multiple times for multiple script lines."
    )]
    pub agent_peer_script_post_up_line: Vec<String>,

    #[arg(long, default_value = None, long_help = "Enable PreDown script for agent"
    )]
    pub agent_peer_script_pre_down_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set PreDown script line(s) for agent. Can be specified multiple times for multiple script lines."
    )]
    pub agent_peer_script_pre_down_line: Vec<String>,

    #[arg(long, default_value = None, long_help = "Enable PostDown script for agent"
    )]
    pub agent_peer_script_post_down_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set PostDown script line(s) for agent. Can be specified multiple times for multiple script lines."
    )]
    pub agent_peer_script_post_down_line: Vec<String>,

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

    #[arg(long, default_value = None, long_help = "Set default DNS addresses for new peers", value_name = "1.1.1.1"
    )]
    pub default_peer_dns_addresses: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable MTU for new peers by default")]
    pub default_peer_mtu_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set default MTU value for new peers", value_name = "1420"
    )]
    pub default_peer_mtu_value: Option<u16>,

    #[arg(long, default_value = None, long_help = "Enable PreUp script for new peers by default"
    )]
    pub default_peer_script_pre_up_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set default PreUp script line(s) for new peers. Can be specified multiple times for multiple script lines."
    )]
    pub default_peer_script_pre_up_line: Vec<String>,

    #[arg(long, default_value = None, long_help = "Enable PostUp script for new peers by default"
    )]
    pub default_peer_script_post_up_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set default PostUp script line(s) for new peers. Can be specified multiple times for multiple script lines."
    )]
    pub default_peer_script_post_up_line: Vec<String>,

    #[arg(long, default_value = None, long_help = "Enable PreDown script for new peers by default"
    )]
    pub default_peer_script_pre_down_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set default PreDown script line(s) for new peers. Can be specified multiple times for multiple script lines."
    )]
    pub default_peer_script_pre_down_line: Vec<String>,

    #[arg(long, default_value = None, long_help = "Enable PostDown script for new peers by default"
    )]
    pub default_peer_script_post_down_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set default PostDown script line(s) for new peers. Can be specified multiple times for multiple script lines."
    )]
    pub default_peer_script_post_down_line: Vec<String>,

    #[arg(long, default_value = None, long_help = "Enable PersistentKeepalive for new connections by default"
    )]
    pub default_connection_persistent_keepalive_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set default PersistentKeepalive period in seconds", value_name = "25"
    )]
    pub default_connection_persistent_keepalive_period: Option<u16>,

    #[arg(long, default_value = None, long_help = "Disable interactive setup prompts")]
    pub no_prompt: Option<bool>,
}

#[derive(Args, Debug)]
pub struct ResetWebPasswordOptions {
    #[arg(long, default_value = None, help = "The use of this option is HIGHLY DISCOURAGED because the plaintext password might show up in the shell history! THIS IS HIGHLY INSECURE! Please set the password without the --password flag, and the script will prompt for the password."
    )]
    pub password: Option<String>,
}
