use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;
use wg_quickrs_cli::Cli;
use clap::{CommandFactory, FromArgMatches};
use once_cell::sync::OnceCell;
use wg_quickrs_lib::validation::error::ValidationError;
use wg_quickrs_lib::macros::full_version;

mod commands;
mod conf;
mod web;
mod wireguard;
mod helpers;

pub static WG_QUICKRS_CONFIG_FOLDER: OnceCell<PathBuf> = OnceCell::new();
pub static WG_QUICKRS_CONFIG_FILE: OnceCell<PathBuf> = OnceCell::new();

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Path Error: {0}")]
    Path(String),
    #[error("{0}")]
    Validation(#[from] ValidationError),
    #[error("{0}")]
    AgentInit(#[from] commands::agent::init::AgentInitError),
    #[error("{0}")]
    AgentRun(#[from] commands::agent::run::AgentRunError),
    #[error("{0}")]
    ConfigCommand(#[from] commands::config::ConfigCommandError),
}

#[actix_web::main]
async fn main() -> ExitCode {
    let matches = Cli::command().version(full_version!()).get_matches();
    let args = Cli::from_arg_matches(&matches).expect("Failed to parse command line arguments");

    // start logger
    SimpleLogger::new()
        .with_level(if args.verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .with_module_level("actix_server", LevelFilter::Warn)
        .init()
        .unwrap_or_else(|e| {
            eprintln!("Logger init failed: {e}");
        });
    log::debug!(full_version!());

    match entrypoint(args).await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            log::error!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn expand_tilde(path: PathBuf) -> PathBuf {
    if let Some(s) = path.to_str()
        && s.starts_with("~")
    {
        let home = dirs::home_dir().expect("Could not get home directory");
        let mut expanded = home;
        expanded.push(s.trim_start_matches("~/"));
        return expanded;
    }
    path
}

async fn entrypoint(args: Cli) -> Result<(), CommandError> {
    // get the wg_quickrs config file path
    let mut config_folder = expand_tilde(args.wg_quickrs_config_folder.clone());
    if !config_folder.exists() {
        log::warn!("config folder does not exist, creating it at \"{}\"", config_folder.display());
        std::fs::create_dir_all(&config_folder)?;
    }
    config_folder = config_folder.canonicalize()?;
    WG_QUICKRS_CONFIG_FOLDER.set(config_folder.clone())
        .map_err(|_| CommandError::Path(format!("Could not set the wg-quickrs config folder to \"{}\"", config_folder.display())))?;
    let mut wg_quickrs_config_file = config_folder;
    wg_quickrs_config_file.push("conf.yml");
    WG_QUICKRS_CONFIG_FILE.set(wg_quickrs_config_file.clone())
        .map_err(|_| CommandError::Path(format!("Could not set the wg-quickrs config file to \"{}\"", wg_quickrs_config_file.display())))?;
    log::debug!("using the wg-quickrs config file at \"{}\"", wg_quickrs_config_file.display());

    match &args.command {
        wg_quickrs_cli::Commands::Agent { target } => {
            match target {
                wg_quickrs_cli::agent::AgentCommands::Init(init_opts) => commands::agent::init::initialize_agent(init_opts)?,
                wg_quickrs_cli::agent::AgentCommands::Run => commands::agent::run::run_agent().await?,
            }
        },
        wg_quickrs_cli::Commands::Config { target } => {
            commands::config::handle_config_command(target)?;
        }
    };

    Ok(())
}

