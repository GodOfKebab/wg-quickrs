use crate::macros::*;
use clap::{ArgGroup, Args, Parser, Subcommand};
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
    #[arg(
        long,
        default_value = ".wg-rusteze/conf.yml",
        value_name = "WG_RUSTEZE_CONFIG_FILE_PATH"
    )]
    pub(crate) wg_rusteze_config_file: PathBuf,
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    #[command(
        about = "Initialize the wg-rusteze rust-agent.\nConfiguration options can be filled either by prompts on screen (when no argument is provided) or specified as arguments to this command"
    )]
    Init {
        #[arg(
            long,
            default_value = None,
            help = "VPN network's identifier (e.g. wg-rusteze)"
        )]
        network_identifier: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "VPN network's CIDR subnet mask (e.g. 10.0.34.0/24)"
        )]
        network_subnet: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Agent's peer name (e.g. wg-rusteze-host)"
        )]
        agent_peer_name: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Agent's local IPv4 address for the web server to bind"
        )]
        agent_local_address: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Agent's local web port for the web server to bind (e.g. 80)"
        )]
        agent_local_web_port: Option<u16>,
        #[arg(
            long,
            default_value = None,
            help = "Agent's local VPN port for the VPN server listen (e.g. 51820)"
        )]
        agent_local_vpn_port: Option<u16>,
        #[arg(
            long,
            default_value = None,
            help = "Agent's publicly accessible IPv4 address to be used in the VPN endpoint advertisement"
        )]
        agent_public_address: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Agent's publicly accessible port to be used in the VPN endpoint advertisement (e.g. 51820)"
        )]
        agent_public_vpn_port: Option<u16>,
        #[arg(
            long,
            default_value = None,
            help = "Agent's internal IPv4 address for VPN network (e.g. 10.0.34.1)"
        )]
        agent_internal_vpn_address: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Enable/Disable TLS for the web server"
        )]
        agent_use_tls: Option<bool>,
        #[arg(
            long,
            default_value = None,
            help = "Enable/Disable password for the web server"
        )]
        agent_enable_web_password: Option<bool>,
        #[arg(
            long,
            default_value = None,
            help = "Agent's password for the web server"
        )]
        agent_web_password: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Enable/Disable DNS field for the agent"
        )]
        agent_enable_dns: Option<bool>,
        #[arg(
            long,
            default_value = None,
            help = "Agent's DNS field setting as an IPv4 address (e.g. 1.1.1.1)"
        )]
        agent_dns_server: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Enable/Disable MTU field for the agent"
        )]
        agent_enable_mtu: Option<bool>,
        #[arg(
            long,
            default_value = None,
            help = "Agent's MTU field setting as a value (e.g. 1420)"
        )]
        agent_mtu_value: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Enable/Disable PreUp scripting field for the agent"
        )]
        agent_enable_script_pre_up: Option<bool>,
        #[arg(
            long,
            default_value = None,
            help = "Agent's PreUp scripting field setting as a line of script (e.g. sysctl -w net.ipv4.ip_forward=1)"
        )]
        agent_script_pre_up_line: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Enable/Disable PostUp scripting field for the agent"
        )]
        agent_enable_script_post_up: Option<bool>,
        #[arg(
            long,
            default_value = None,
            help = "Agent's PostUp scripting field setting as a line of script (e.g. echo 'Network is up')"
        )]
        agent_script_post_up_line: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Enable/Disable PreDown scripting field for the agent"
        )]
        agent_enable_script_pre_down: Option<bool>,
        #[arg(
            long,
            default_value = None,
            help = "Agent's PreDown scripting field setting as a line of script (e.g. echo 'Network is going down')"
        )]
        agent_script_pre_down_line: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Enable/Disable PostDown scripting field for the agent"
        )]
        agent_enable_script_post_down: Option<bool>,
        #[arg(
            long,
            default_value = None,
            help = "Agent's PostDown scripting field setting as a line of script (e.g. sysctl -w net.ipv4.ip_forward=0)"
        )]
        agent_script_post_down_line: Option<String>,
        // default settings for new peers/connections
        #[arg(
            long,
            default_value = None,
            help = "Enable/Disable DNS field for new peers by default"
        )]
        default_enable_dns: Option<bool>,
        #[arg(
            long,
            default_value = None,
            help = "Default DNS field setting as an IPv4 address for the new peer (e.g. 1.1.1.1)"
        )]
        default_dns_server: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Enable/Disable MTU field for new peers by default"
        )]
        default_enable_mtu: Option<bool>,
        #[arg(
            long,
            default_value = None,
            help = "Default MTU field setting as a value for the new peer (e.g. 1420)"
        )]
        default_mtu_value: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Enable/Disable PreUp scripting field for new peers by default"
        )]
        default_enable_script_pre_up: Option<bool>,
        #[arg(
            long,
            default_value = None,
            help = "New Peer's default PreUp scripting field setting as a line of script (e.g. sysctl -w net.ipv4.ip_forward=1)"
        )]
        default_script_pre_up_line: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Enable/Disable PostUp scripting field for new peers by default"
        )]
        default_enable_script_post_up: Option<bool>,
        #[arg(
            long,
            default_value = None,
            help = "New Peer's default PostUp scripting field setting as a line of script (e.g. echo 'Network is up')"
        )]
        default_script_post_up_line: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Enable/Disable PreDown scripting field for new peers by default"
        )]
        default_enable_script_pre_down: Option<bool>,
        #[arg(
            long,
            default_value = None,
            help = "New Peer's default PreDown scripting field setting as a line of script (e.g. echo 'Network is going down')"
        )]
        default_script_pre_down_line: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Enable/Disable PostDown scripting field for the agent"
        )]
        default_enable_script_post_down: Option<bool>,
        #[arg(
            long,
            default_value = None,
            help = "Agent's PostDown scripting field setting as a line of script (e.g. sysctl -w net.ipv4.ip_forward=0)"
        )]
        default_script_post_down_line: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Enable/Disable PersistentKeepalive field for new connections by default"
        )]
        default_enable_persistent_keepalive: Option<bool>,
        #[arg(
            long,
            default_value = None,
            help = "Default PersistentKeepalive period in seconds (e.g. 25)"
        )]
        default_persistent_keepalive_period: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Disable setting up the agent with prompts"
        )]
        no_prompt: Option<bool>,
    },
    #[command(about = "Run some convenience functions to edit config")]
    Config {
        #[command(subcommand)]
        commands: ConfigCommands,
    },
    #[command(about = "Configure and run the wg-rusteze rust-agent")]
    Agent {
        #[arg(
            long,
            default_value = "/opt/homebrew/etc/wireguard/",
            value_name = "WIREGUARD_CONFIG_FOLDER_PATH"
        )]
        wireguard_config_folder: PathBuf,
        #[arg(
            long,
            default_value = ".wg-rusteze/cert.pem",
            value_name = "TLS_CERTIFICATE_FILE_PATH"
        )]
        tls_cert: PathBuf,
        #[arg(
            long,
            default_value = ".wg-rusteze/key.pem",
            value_name = "TLS_PRIVATE_KEY_FILE_PATH"
        )]
        tls_key: PathBuf,
        #[command(subcommand)]
        commands: AgentCommands,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum AgentCommands {
    #[command(about = "Runs the rust-agent")]
    Run(AgentRunOptions),
}

#[derive(Args, Debug)]
#[command(group(
    ArgGroup::new("mode")
        .args(&["all", "only_web", "only_wireguard"])
        .multiple(false)
        .required(false)
))]
pub(crate) struct AgentRunOptions {
    #[arg(
        long,
        help = "Start the wireguard server and run the web configuration portal"
    )]
    pub(crate) all: bool,
    #[arg(long, help = "Run only the web configuration portal")]
    pub(crate) only_web: bool,
    #[arg(long, help = "Start only the wireguard server")]
    pub(crate) only_wireguard: bool,
}

#[derive(Subcommand, Debug)]
pub(crate) enum ConfigCommands {
    #[command(about = "Reset the web password")]
    ResetWebPassword,
}
