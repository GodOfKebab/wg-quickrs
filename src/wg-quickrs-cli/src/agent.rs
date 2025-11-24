use std::net::Ipv4Addr;
use std::path::PathBuf;
use clap::{Args, Subcommand};
use ipnet::Ipv4Net;

#[derive(Subcommand, Debug)]
pub enum AgentCommands {
    #[command(
        about = "Initialize the wg-quickrs agent.\nConfiguration options can be filled either by prompts on screen (when no argument is provided) or specified as arguments to this command"
    )]
    Init(Box<InitOptions>),
    #[command(about = "Run the wg-quickrs agent")]
    Run,
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

    #[arg(long, default_value = None, long_help = "Configure HTTP firewall")]
    pub agent_firewall_configure_http: Option<bool>,

    #[arg(long, default_value = None, long_help = "Use automated setup for HTTP firewall")]
    pub agent_firewall_http_automated: Option<bool>,

    #[arg(long, default_value = None, long_help = "Configure HTTPS firewall")]
    pub agent_firewall_configure_https: Option<bool>,

    #[arg(long, default_value = None, long_help = "Use automated setup for HTTPS firewall")]
    pub agent_firewall_https_automated: Option<bool>,

    #[arg(long, default_value = None, long_help = "Configure VPN firewall")]
    pub agent_firewall_configure_vpn: Option<bool>,

    #[arg(long, default_value = None, long_help = "Use automated setup for VPN firewall")]
    pub agent_firewall_vpn_automated: Option<bool>,

    // HTTP firewall scripts
    #[arg(long, default_value = None, long_help = "Enable HTTP firewall PreUp scripts")]
    pub agent_firewall_http_pre_up_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set HTTP firewall PreUp script line(s). Can be specified multiple times for multiple script lines.")]
    pub agent_firewall_http_pre_up_line: Vec<String>,

    #[arg(long, default_value = None, long_help = "Enable HTTP firewall PostDown scripts")]
    pub agent_firewall_http_post_down_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set HTTP firewall PostDown script line(s). Can be specified multiple times for multiple script lines.")]
    pub agent_firewall_http_post_down_line: Vec<String>,

    // HTTPS firewall scripts
    #[arg(long, default_value = None, long_help = "Enable HTTPS firewall PreUp scripts")]
    pub agent_firewall_https_pre_up_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set HTTPS firewall PreUp script line(s). Can be specified multiple times for multiple script lines.")]
    pub agent_firewall_https_pre_up_line: Vec<String>,

    #[arg(long, default_value = None, long_help = "Enable HTTPS firewall PostDown scripts")]
    pub agent_firewall_https_post_down_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set HTTPS firewall PostDown script line(s). Can be specified multiple times for multiple script lines.")]
    pub agent_firewall_https_post_down_line: Vec<String>,

    // VPN firewall scripts
    #[arg(long, default_value = None, long_help = "Enable VPN firewall PreUp scripts")]
    pub agent_firewall_vpn_pre_up_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set VPN firewall PreUp script line(s). Can be specified multiple times for multiple script lines.")]
    pub agent_firewall_vpn_pre_up_line: Vec<String>,

    #[arg(long, default_value = None, long_help = "Enable VPN firewall PostUp scripts")]
    pub agent_firewall_vpn_post_up_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set VPN firewall PostUp script line(s). Can be specified multiple times for multiple script lines.")]
    pub agent_firewall_vpn_post_up_line: Vec<String>,

    #[arg(long, default_value = None, long_help = "Enable VPN firewall PreDown scripts")]
    pub agent_firewall_vpn_pre_down_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set VPN firewall PreDown script line(s). Can be specified multiple times for multiple script lines.")]
    pub agent_firewall_vpn_pre_down_line: Vec<String>,

    #[arg(long, default_value = None, long_help = "Enable VPN firewall PostDown scripts")]
    pub agent_firewall_vpn_post_down_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set VPN firewall PostDown script line(s). Can be specified multiple times for multiple script lines.")]
    pub agent_firewall_vpn_post_down_line: Vec<String>,

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

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set DNS address for agent", value_name = "1.1.1.1"
    )]
    pub agent_peer_dns_addresses: Vec<Ipv4Addr>,

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

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set default DNS address for new peers", value_name = "1.1.1.1"
    )]
    pub default_peer_dns_addresses: Vec<Ipv4Addr>,

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
