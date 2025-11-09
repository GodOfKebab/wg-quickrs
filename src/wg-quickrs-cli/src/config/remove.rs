use std::net::Ipv4Addr;
use clap::Subcommand;
use uuid::Uuid;

#[derive(Subcommand, Debug)]
pub enum RemoveCommands {
    #[command(about = "Remove a peer by UUID")]
    Peer {
        #[arg(help = "Peer UUID to remove")]
        id: Uuid,
    },
    #[command(about = "Remove a connection by connection ID")]
    Connection {
        #[arg(help = "Connection ID (format: uuid*uuid)")]
        id: String,
    },
    #[command(about = "Remove a reservation by IPv4 address")]
    Reservation {
        #[arg(help = "IPv4 address of the reservation to remove")]
        address: Ipv4Addr,
    },
}
