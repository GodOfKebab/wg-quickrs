use std::net::Ipv4Addr;
use clap::Subcommand;
use uuid::Uuid;

#[derive(Subcommand, Debug)]
pub enum GetCommands {
    #[command(about = "Get agent configuration values")]
    Agent {
        #[command(subcommand)]
        target: Option<GetAgentCommands>,
    },
    #[command(about = "Get network configuration values")]
    Network {
        #[command(subcommand)]
        target: Option<GetNetworkCommands>,
    },
}

#[derive(Subcommand, Debug)]
pub enum GetAgentCommands {
    #[command(about = "Get web server configuration")]
    Web {
        #[command(subcommand)]
        target: Option<GetAgentWebCommands>,
    },
    #[command(about = "Get VPN configuration")]
    Vpn {
        #[command(subcommand)]
        target: Option<GetAgentVpnCommands>,
    },
    #[command(about = "Get firewall configuration")]
    Firewall {
        #[command(subcommand)]
        target: Option<GetAgentFirewallCommands>,
    },
}

#[derive(Subcommand, Debug)]
pub enum GetAgentWebCommands {
    #[command(about = "Get agent web server bind IPv4 address")]
    Address,
    #[command(about = "Get HTTP configuration")]
    Http {
        #[command(subcommand)]
        target: Option<GetAgentWebHttpCommands>,
    },
    #[command(about = "Get HTTPS configuration")]
    Https {
        #[command(subcommand)]
        target: Option<GetAgentWebHttpsCommands>,
    },
    #[command(about = "Get password authentication configuration")]
    Password {
        #[command(subcommand)]
        target: Option<GetAgentWebPasswordCommands>,
    },
}

#[derive(Subcommand, Debug)]
pub enum GetAgentWebHttpCommands {
    #[command(about = "Get whether HTTP is enabled")]
    Enabled,
    #[command(about = "Get web server HTTP port")]
    Port,
}

#[derive(Subcommand, Debug)]
pub enum GetAgentWebHttpsCommands {
    #[command(about = "Get whether HTTPS is enabled")]
    Enabled,
    #[command(about = "Get web server HTTPS port")]
    Port,
    #[command(about = "Get path to TLS certificate file for HTTPS")]
    TlsCert,
    #[command(about = "Get path to TLS private key file for HTTPS")]
    TlsKey,
}

#[derive(Subcommand, Debug)]
pub enum GetAgentWebPasswordCommands {
    #[command(about = "Get whether password authentication is enabled")]
    Enabled,
    #[command(about = "Get password hash")]
    Hash,
}

#[derive(Subcommand, Debug)]
pub enum GetAgentVpnCommands {
    #[command(about = "Get whether VPN server is enabled")]
    Enabled,
    #[command(about = "Get VPN server listening port")]
    Port,
}

#[derive(Subcommand, Debug)]
pub enum GetAgentFirewallCommands {
    #[command(about = "Get whether firewall configuration is enabled")]
    Enabled,
    #[command(about = "Get the utility used to configure firewall NAT and input rules")]
    Utility,
    #[command(about = "Get the gateway used to configure firewall NAT and input rules")]
    Gateway,
}

#[derive(Subcommand, Debug)]
pub enum GetNetworkCommands {
    #[command(about = "Get network name")]
    Name,
    #[command(about = "Get network subnet")]
    Subnet,
    #[command(about = "Get this peer's UUID")]
    ThisPeer,
    #[command(about = "Get network peers")]
    Peers {
        #[arg(help = "Peer UUID")]
        id: Option<Uuid>,
        #[command(subcommand)]
        target: Option<GetNetworkPeersCommands>,
    },
    #[command(about = "Get network connections")]
    Connections {
        #[arg(help = "Connection ID (format: uuid*uuid)")]
        id: Option<String>,
        #[command(subcommand)]
        target: Option<GetNetworkConnectionsCommands>,
    },
    #[command(about = "Get network defaults")]
    Defaults {
        #[command(subcommand)]
        target: Option<GetNetworkDefaultsCommands>,
    },
    #[command(about = "Get network reservations")]
    Reservations {
        #[arg(help = "IPv4 address")]
        ip: Option<Ipv4Addr>,
        #[command(subcommand)]
        target: Option<GetNetworkReservationsCommands>,
    },
    #[command(about = "Get network last updated timestamp")]
    UpdatedAt,
}

#[derive(Subcommand, Debug)]
pub enum GetNetworkPeersCommands {
    #[command(about = "Get peer name")]
    Name,
    #[command(about = "Get peer IP address")]
    Address,
    #[command(about = "Get peer endpoint")]
    Endpoint {
        #[command(subcommand)]
        target: Option<GetNetworkPeersEndpointCommands>,
    },
    #[command(about = "Get peer kind")]
    Kind,
    #[command(about = "Get peer icon")]
    Icon {
        #[command(subcommand)]
        target: Option<GetNetworkPeersIconCommands>,
    },
    #[command(about = "Get peer DNS")]
    Dns {
        #[command(subcommand)]
        target: Option<GetNetworkPeersDnsCommands>,
    },
    #[command(about = "Get peer MTU")]
    Mtu {
        #[command(subcommand)]
        target: Option<GetNetworkPeersMtuCommands>,
    },
    #[command(about = "Get peer scripts")]
    Scripts,
    #[command(about = "Get peer private key")]
    PrivateKey,
    #[command(about = "Get peer creation timestamp")]
    CreatedAt,
    #[command(about = "Get peer last updated timestamp")]
    UpdatedAt,
}

#[derive(Subcommand, Debug)]
pub enum GetNetworkPeersEndpointCommands {
    #[command(about = "Get whether endpoint is enabled")]
    Enabled,
    #[command(about = "Get endpoint address")]
    Address,
}

#[derive(Subcommand, Debug)]
pub enum GetNetworkPeersIconCommands {
    #[command(about = "Get whether icon is enabled")]
    Enabled,
    #[command(about = "Get icon source")]
    Src,
}

#[derive(Subcommand, Debug)]
pub enum GetNetworkPeersDnsCommands {
    #[command(about = "Get whether DNS is enabled")]
    Enabled,
    #[command(about = "Get DNS addresses")]
    Addresses,
}

#[derive(Subcommand, Debug)]
pub enum GetNetworkPeersMtuCommands {
    #[command(about = "Get whether MTU is enabled")]
    Enabled,
    #[command(about = "Get MTU value")]
    Value,
}

#[derive(Subcommand, Debug)]
pub enum GetNetworkConnectionsCommands {
    #[command(about = "Get whether connection is enabled")]
    Enabled,
    #[command(about = "Get connection pre-shared key")]
    PreSharedKey,
    #[command(about = "Get connection persistent keepalive")]
    PersistentKeepalive {
        #[command(subcommand)]
        target: Option<GetNetworkConnectionsPersistentKeepaliveCommands>,
    },
    #[command(about = "Get allowed IPs from A to B")]
    AllowedIpsAToB,
    #[command(about = "Get allowed IPs from B to A")]
    AllowedIpsBToA,
}

#[derive(Subcommand, Debug)]
pub enum GetNetworkConnectionsPersistentKeepaliveCommands {
    #[command(about = "Get whether persistent keepalive is enabled")]
    Enabled,
    #[command(about = "Get persistent keepalive period")]
    Period,
}

#[derive(Subcommand, Debug)]
pub enum GetNetworkDefaultsCommands {
    #[command(about = "Get default peer configuration")]
    Peer {
        #[command(subcommand)]
        target: Option<GetNetworkDefaultsPeerCommands>,
    },
    #[command(about = "Get default connection configuration")]
    Connection {
        #[command(subcommand)]
        target: Option<GetNetworkDefaultsConnectionCommands>,
    },
}

#[derive(Subcommand, Debug)]
pub enum GetNetworkDefaultsPeerCommands {
    #[command(about = "Get default peer kind")]
    Kind,
    #[command(about = "Get default peer icon")]
    Icon {
        #[command(subcommand)]
        target: Option<GetNetworkPeersIconCommands>,
    },
    #[command(about = "Get default peer DNS")]
    Dns {
        #[command(subcommand)]
        target: Option<GetNetworkPeersDnsCommands>,
    },
    #[command(about = "Get default peer MTU")]
    Mtu {
        #[command(subcommand)]
        target: Option<GetNetworkPeersMtuCommands>,
    },
    #[command(about = "Get default peer scripts")]
    Scripts,
}

#[derive(Subcommand, Debug)]
pub enum GetNetworkDefaultsConnectionCommands {
    #[command(about = "Get default connection persistent keepalive")]
    PersistentKeepalive {
        #[command(subcommand)]
        target: Option<GetNetworkConnectionsPersistentKeepaliveCommands>,
    },
}

#[derive(Subcommand, Debug)]
pub enum GetNetworkReservationsCommands {
    #[command(about = "Get reservation peer ID")]
    PeerId,
    #[command(about = "Get reservation validity timestamp")]
    ValidUntil,
}
