pub mod enable;
pub mod disable;
pub mod set;
pub mod reset;
pub mod get;
pub mod list;
pub mod remove;
pub mod add;
pub mod conf;

use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    #[command(about = "Enable a configuration option")]
    Enable {
        #[command(subcommand)]
        target: enable::EnableCommands,
    },
    #[command(about = "Disable a configuration option")]
    Disable {
        #[command(subcommand)]
        target: disable::DisableCommands,
    },
    #[command(about = "Set a configuration value")]
    Set {
        #[command(subcommand)]
        target: set::SetCommands,
    },
    #[command(about = "Reset a configuration option")]
    Reset {
        #[command(subcommand)]
        target: reset::ResetCommands,
    },
    #[command(about = "Get a configuration value")]
    Get {
        #[command(subcommand)]
        target: get::GetCommands,
    },
    #[command(about = "List network entities in human-readable format")]
    List {
        #[command(subcommand)]
        target: list::ListCommands,
    },
    #[command(about = "Remove network entities")]
    Remove {
        #[command(subcommand)]
        target: remove::RemoveCommands,
    },
    #[command(about = "Add network entities")]
    Add {
        #[command(subcommand)]
        target: add::AddCommands,
    },
    #[command(
        about = "Generate wg/awg or wg-quick/awg-quick configuration file for a peer",
    )]
    Conf {
        #[command(flatten)]
        options: conf::ConfOptions,
    },
}
