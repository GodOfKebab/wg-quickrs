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
            help = "Network identifier (e.g. wg-rusteze)"
        )]
        network_identifier: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Peer name (e.g. wg-rusteze-host)"
        )]
        peer_name: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Public IPv4 address for rust-agent"
        )]
        public_address: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "HTTP(S) port for rust-agent (e.g. 80)"
        )]
        web_port: Option<u16>,
        #[arg(
            long,
            default_value = None,
            help = "VPN port for rust-agent (e.g. 51820)"
        )]
        vpn_port: Option<u16>,
        #[arg(
            long,
            default_value = None,
            help = "CIDR subnet mask for VPN network (e.g. 10.0.34.0/24)"
        )]
        subnet: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Internal IPv4 address for VPN network (e.g. 10.0.34.1)"
        )]
        vpn_address: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "Enable/Disable TLS for HTTP server"
        )]
        use_tls: Option<bool>,
        #[arg(
            long,
            default_value = None,
            help = "Password for HTTP server"
        )]
        password: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "DNS IPv4 address for VPN network (e.g. 1.1.1.1)"
        )]
        dns_server: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "MTU for VPN network (e.g. 1420)"
        )]
        mtu_value: Option<String>,
        #[arg(
            long,
            default_value = None,
            help = "PersistentKeepalive value in seconds (e.g. 25)"
        )]
        persistent_keepalive_seconds: Option<String>,
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
