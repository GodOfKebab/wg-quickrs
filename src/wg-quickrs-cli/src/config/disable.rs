use clap::Subcommand;
use uuid::Uuid;

#[derive(Subcommand, Debug)]
pub enum DisableCommands {
    #[command(about = "Disable agent configuration options")]
    Agent {
        #[command(subcommand)]
        target: DisableAgentCommands,
    },
    #[command(about = "Disable network configuration options")]
    Network {
        #[command(subcommand)]
        target: DisableNetworkCommands,
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
pub enum DisableNetworkCommands {
    #[command(about = "Disable peer options")]
    Peer {
        #[arg(help = "Peer UUID")]
        id: Uuid,
        #[command(subcommand)]
        target: DisablePeerCommands,
    },
    #[command(about = "Disable connection")]
    Connection {
        #[arg(help = "Connection ID (format: uuid*uuid)")]
        id: String,
    },
    #[command(about = "Disable default configuration options")]
    Defaults {
        #[command(subcommand)]
        target: DisableDefaultsCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum DisablePeerCommands {
    #[command(about = "Disable peer endpoint")]
    Endpoint,
    #[command(about = "Disable peer icon")]
    Icon,
    #[command(about = "Disable peer DNS")]
    Dns,
    #[command(about = "Disable peer MTU")]
    Mtu,
}

#[derive(Subcommand, Debug)]
pub enum DisableDefaultsCommands {
    #[command(about = "Disable default peer options")]
    Peer {
        #[command(subcommand)]
        target: DisableDefaultsPeerCommands,
    },
    #[command(about = "Disable default connection options")]
    Connection {
        #[command(subcommand)]
        target: DisableDefaultsConnectionCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum DisableDefaultsPeerCommands {
    #[command(about = "Disable default peer endpoint")]
    Endpoint,
    #[command(about = "Disable default peer icon")]
    Icon,
    #[command(about = "Disable default peer DNS")]
    Dns,
    #[command(about = "Disable default peer MTU")]
    Mtu,
}

#[derive(Subcommand, Debug)]
pub enum DisableDefaultsConnectionCommands {
    #[command(about = "Disable default connection persistent keepalive")]
    PersistentKeepalive,
}
