use crate::commands::config::{ConfigCommandError};
use log::{LevelFilter};
use wg_quickrs_cli::Cli;
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;
use wg_quickrs::{WG_QUICKRS_CONFIG_FOLDER, WG_QUICKRS_CONFIG_FILE};
use wg_quickrs_lib::validation::error::ValidationError;
use wg_quickrs_lib::macros::full_version;
use crate::commands::agent::AgentRunError;
use crate::commands::init::InitError;


mod cli;
mod commands;
mod conf;
mod web;
mod wireguard;


#[derive(Error, Debug)]
pub enum CommandError {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Path Error: {0}")]
    Path(String),
    #[error("{0}")]
    Validation(#[from] ValidationError),
    #[error("{0}")]
    AgentRun(#[from] AgentRunError),
    #[error("{0}")]
    Init(#[from] InitError),
    #[error("{0}")]
    ConfigCommand(#[from] ConfigCommandError),
}

#[actix_web::main]
async fn main() -> ExitCode {
    let args = cli::parse();
    println!(full_version!());

    // start logger
    SimpleLogger::new()
        .with_level(if args.verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .init()
        .unwrap_or_else(|e| {
            eprintln!("Logger init failed: {e}");
        });

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
    log::info!("using the wg-quickrs config file at \"{}\"", wg_quickrs_config_file.display());

    match &args.command {
        wg_quickrs_cli::Commands::Agent { target } => {
            match target {
                wg_quickrs_cli::AgentCommands::Init(init_opts) => commands::init::initialize_agent(init_opts)?,
                wg_quickrs_cli::AgentCommands::Run => commands::agent::run_agent().await?,
            }
        },
        wg_quickrs_cli::Commands::Config { target } => {
            match target {
                wg_quickrs_cli::ConfigCommands::Enable { target } => {
                    match target {
                        wg_quickrs_cli::EnableCommands::Agent { target } => {
                            match target {
                                wg_quickrs_cli::EnableAgentCommands::Web { target } => {
                                    match target {
                                        wg_quickrs_cli::EnableAgentWebCommands::Http => commands::config::toggle_agent_web_http(true)?,
                                        wg_quickrs_cli::EnableAgentWebCommands::Https => commands::config::toggle_agent_web_https(true)?,
                                        wg_quickrs_cli::EnableAgentWebCommands::Password => commands::config::toggle_agent_web_password(true)?,
                                    }
                                },
                                wg_quickrs_cli::EnableAgentCommands::Vpn => commands::config::toggle_agent_vpn(true)?,
                                wg_quickrs_cli::EnableAgentCommands::Firewall => commands::config::toggle_agent_firewall(true)?,
                            }
                        },
                    }
                },
                wg_quickrs_cli::ConfigCommands::Disable { target } => {
                    match target {
                        wg_quickrs_cli::DisableCommands::Agent { target } => {
                            match target {
                                wg_quickrs_cli::DisableAgentCommands::Web { target } => {
                                    match target {
                                        wg_quickrs_cli::DisableAgentWebCommands::Http => commands::config::toggle_agent_web_http(false)?,
                                        wg_quickrs_cli::DisableAgentWebCommands::Https => commands::config::toggle_agent_web_https(false)?,
                                        wg_quickrs_cli::DisableAgentWebCommands::Password => commands::config::toggle_agent_web_password(false)?,
                                    }
                                },
                                wg_quickrs_cli::DisableAgentCommands::Vpn => commands::config::toggle_agent_vpn(false)?,
                                wg_quickrs_cli::DisableAgentCommands::Firewall => commands::config::toggle_agent_firewall(false)?,
                            }
                        },
                    }
                },
                wg_quickrs_cli::ConfigCommands::Set { target } => {
                    match target {
                        wg_quickrs_cli::SetCommands::Agent { target } => {
                            match target {
                                wg_quickrs_cli::SetAgentCommands::Web { target } => {
                                    match target {
                                        wg_quickrs_cli::SetAgentWebCommands::Address { value } => commands::config::set_agent_web_address(value)?,
                                        wg_quickrs_cli::SetAgentWebCommands::Http { target } => {
                                            match target {
                                                wg_quickrs_cli::SetAgentWebHttpCommands::Port { value } => commands::config::set_agent_web_http_port(*value)?,
                                            }
                                        },
                                        wg_quickrs_cli::SetAgentWebCommands::Https { target } => {
                                            match target {
                                                wg_quickrs_cli::SetAgentWebHttpsCommands::Port { value } => commands::config::set_agent_web_https_port(*value)?,
                                                wg_quickrs_cli::SetAgentWebHttpsCommands::TlsCert { value } => commands::config::set_agent_web_http_tls_cert(value)?,
                                                wg_quickrs_cli::SetAgentWebHttpsCommands::TlsKey { value } => commands::config::set_agent_web_http_tls_key(value)?,
                                            }
                                        },
                                    }
                                },
                                wg_quickrs_cli::SetAgentCommands::Vpn { target } => {
                                    match target {
                                        wg_quickrs_cli::SetAgentVpnCommands::Port { value } => commands::config::set_agent_vpn_port(*value)?,
                                    }
                                },
                                wg_quickrs_cli::SetAgentCommands::Firewall { target } => {
                                    match target {
                                        wg_quickrs_cli::SetAgentFirewallCommands::Utility { value } => commands::config::set_agent_firewall_utility(value)?,
                                        wg_quickrs_cli::SetAgentFirewallCommands::Gateway { value } => commands::config::set_agent_firewall_gateway(value)?,
                                    }
                                },
                            }
                        },
                    }
                },
                wg_quickrs_cli::ConfigCommands::Reset { target } => {
                    match target {
                        wg_quickrs_cli::ResetCommands::Agent { target } => {
                            match target {
                                wg_quickrs_cli::ResetAgentCommands::Web { target } => {
                                    match target {
                                        wg_quickrs_cli::ResetAgentWebCommands::Password { password } => {
                                            commands::config::reset_web_password(&wg_quickrs_cli::ResetWebPasswordOptions {
                                                password: password.clone(),
                                            })?;
                                        },
                                    }
                                },
                            }
                        },
                    }
                },
            }
        }
    };

    Ok(())
}

