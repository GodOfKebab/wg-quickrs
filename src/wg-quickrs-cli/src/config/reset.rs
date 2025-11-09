use clap::Subcommand;
use uuid::Uuid;

#[derive(Subcommand, Debug)]
pub enum ResetCommands {
    #[command(about = "Reset agent configuration options")]
    Agent {
        #[command(subcommand)]
        target: ResetAgentCommands,
    },
    #[command(about = "Reset network configuration options")]
    Network {
        #[command(subcommand)]
        target: ResetNetworkCommands,
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

#[derive(Subcommand, Debug)]
pub enum ResetNetworkCommands {
    #[command(about = "Reset peer options")]
    Peer {
        #[arg(help = "Peer UUID")]
        id: Uuid,
        #[command(subcommand)]
        target: ResetPeerCommands,
    },
    #[command(about = "Reset connection options")]
    Connection {
        #[arg(help = "Connection ID (format: uuid*uuid)")]
        id: String,
        #[command(subcommand)]
        target: ResetConnectionCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ResetPeerCommands {
    #[command(about = "Reset peer private key (generates new WireGuard key)")]
    PrivateKey,
}

#[derive(Subcommand, Debug)]
pub enum ResetConnectionCommands {
    #[command(about = "Reset connection pre-shared key (generates new WireGuard key)")]
    PreSharedKey,
}
