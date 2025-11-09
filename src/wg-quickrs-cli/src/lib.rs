pub mod agent;
pub mod config;

use clap::{Parser, Subcommand};
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
        target: agent::AgentCommands,
    },
    #[command(about = "Edit agent configuration options")]
    Config {
        #[command(subcommand)]
        target: config::ConfigCommands,
    },
}
