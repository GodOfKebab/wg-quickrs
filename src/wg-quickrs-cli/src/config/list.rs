use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum ListCommands {
    #[command(about = "List all peers in human-readable format")]
    Peers,
    #[command(about = "List all connections in human-readable format")]
    Connections,
    #[command(about = "List all reservations in human-readable format")]
    Reservations,
}
