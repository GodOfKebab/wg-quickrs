use clap::Args;
use uuid::Uuid;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct ConfOptions {
    #[arg(help = "Peer ID to generate WireGuard configuration for", long_help = "UUID of the peer to generate WireGuard configuration for. Use 'wg-quickrs config list peers' to see available peer IDs.")]
    pub peer_id: Uuid,

    #[arg(long, help = "Generate stripped config", long_help = "Use --stripped to use with wg/awg, otherwise the conf will only be valid for wg-quick/awg-quick.")]
    pub stripped: bool,

    #[arg(short, long, value_name = "FILE", help = "Output file path", long_help = "Write the generated WireGuard configuration to the specified file path. If not specified, the configuration will be written to stdout.")]
    pub out: Option<PathBuf>,
}
