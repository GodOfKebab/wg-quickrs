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
    #[command(about = "Get path to WireGuard binary")]
    Wg,
    #[command(about = "Get WireGuard userspace configuration")]
    WgUserspace {
        #[command(subcommand)]
        target: Option<GetAgentVpnWgUserspaceCommands>,
    },
}

#[derive(Subcommand, Debug)]
pub enum GetAgentVpnWgUserspaceCommands {
    #[command(about = "Get whether WireGuard userspace is enabled")]
    Enabled,
    #[command(about = "Get path to WireGuard userspace binary")]
    Binary,
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
    #[command(about = "Get AmneziaWG network parameters")]
    AmneziaParameters {
        #[command(subcommand)]
        target: Option<GetNetworkAmneziaParametersCommands>,
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
    #[command(about = "Get peer AmneziaWG parameters")]
    AmneziaParameters {
        #[command(subcommand)]
        target: Option<GetNetworkPeersAmneziaParametersCommands>,
    },
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
pub enum GetNetworkPeersAmneziaParametersCommands {
    #[command(about = "Get Jc parameter (junk packet count)")]
    Jc,
    #[command(about = "Get Jmin parameter (minimum junk packet size)")]
    Jmin,
    #[command(about = "Get Jmax parameter (maximum junk packet size)")]
    Jmax,
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
    #[command(about = "Get default peer AmneziaWG parameters")]
    AmneziaParameters {
        #[command(subcommand)]
        target: Option<GetNetworkPeersAmneziaParametersCommands>,
    },
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

#[derive(Subcommand, Debug)]
pub enum GetNetworkAmneziaParametersCommands {
    #[command(about = "Get whether AmneziaWG obfuscation is enabled")]
    Enabled,
    #[command(about = "Get S1 parameter (init packet junk size)")]
    S1,
    #[command(about = "Get S2 parameter (response packet junk size)")]
    S2,
    #[command(about = "Get H1 parameter (init packet magic header)")]
    H1,
    #[command(about = "Get H2 parameter (response packet magic header)")]
    H2,
    #[command(about = "Get H3 parameter (underload packet magic header)")]
    H3,
    #[command(about = "Get H4 parameter (transport packet magic header)")]
    H4,
}
