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
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    #[command(about = "Initialize the wg-rusteze agent")]
    Init {},
    #[command(about = "Run some convenience functions to edit config")]
    Config {
        #[arg(
            long,
            default_value = ".wg-rusteze/conf.yml",
            value_name = "WG_RUSTEZE_CONFIG_FILE_PATH"
        )]
        wg_rusteze_config_file: PathBuf,
        #[command(subcommand)]
        commands: ConfigCommands,
    },
    #[command(about = "Configure and run the wg-rusteze agent")]
    Agent {
        #[arg(
            long,
            default_value = ".wg-rusteze/conf.yml",
            value_name = "WG_RUSTEZE_CONFIG_FILE_PATH"
        )]
        wg_rusteze_config_file: PathBuf,
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
    #[command(about = "Runs the agent")]
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
