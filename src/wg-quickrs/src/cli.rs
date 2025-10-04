use crate::macros::full_version;
use clap::{CommandFactory, FromArgMatches};
use wg_quickrs_cli::Cli;

// Create a new Cli instance with the version set from the full_version!() macro
pub(crate) fn parse() -> Cli {
    // Set the version at runtime using the full_version!() macro
    let matches = Cli::command().version(full_version!()).get_matches();

    // Parse the command line arguments using the matches
    Cli::from_arg_matches(&matches).expect("Failed to parse command line arguments")
}

// All CLI definitions are now imported from wg-quickrs-cli crate
