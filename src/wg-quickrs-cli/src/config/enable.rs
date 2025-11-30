use clap::Subcommand;
use uuid::Uuid;

#[derive(Subcommand, Debug)]
pub enum EnableCommands {
    #[command(about = "Enable agent configuration options")]
    Agent {
        #[command(subcommand)]
        target: EnableAgentCommands,
    },
    #[command(about = "Enable network configuration options")]
    Network {
        #[command(subcommand)]
        target: EnableNetworkCommands,
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
    Vpn {
        #[command(subcommand)]
        target: Option<EnableAgentVpnCommands>,
    },
}

#[derive(Subcommand, Debug)]
pub enum EnableAgentVpnCommands {
    #[command(about = "Enable WireGuard userspace mode")]
    WgUserspace,
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
pub enum EnableNetworkCommands {
    #[command(about = "Enable peer options")]
    Peer {
        #[arg(help = "Peer UUID")]
        id: Uuid,
        #[command(subcommand)]
        target: EnablePeerCommands,
    },
    #[command(about = "Enable connection")]
    Connection {
        #[arg(help = "Connection ID (format: uuid*uuid)")]
        id: String,
    },
    #[command(about = "Enable default configuration options")]
    Defaults {
        #[command(subcommand)]
        target: EnableDefaultsCommands,
    },
    #[command(about = "Enable AmneziaWG obfuscation")]
    AmneziaParameters,
}

#[derive(Subcommand, Debug)]
pub enum EnablePeerCommands {
    #[command(about = "Enable peer endpoint")]
    Endpoint,
    #[command(about = "Enable peer icon")]
    Icon,
    #[command(about = "Enable peer DNS")]
    Dns,
    #[command(about = "Enable peer MTU")]
    Mtu,
}

#[derive(Subcommand, Debug)]
pub enum EnableDefaultsCommands {
    #[command(about = "Enable default peer options")]
    Peer {
        #[command(subcommand)]
        target: EnableDefaultsPeerCommands,
    },
    #[command(about = "Enable default connection options")]
    Connection {
        #[command(subcommand)]
        target: EnableDefaultsConnectionCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum EnableDefaultsPeerCommands {
    #[command(about = "Enable default peer icon")]
    Icon,
    #[command(about = "Enable default peer DNS")]
    Dns,
    #[command(about = "Enable default peer MTU")]
    Mtu,
}

#[derive(Subcommand, Debug)]
pub enum EnableDefaultsConnectionCommands {
    #[command(about = "Enable default connection persistent keepalive")]
    PersistentKeepalive,
}
