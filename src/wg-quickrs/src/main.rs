use crate::commands::config::{ConfigCommandError};
use log::{LevelFilter};
use wg_quickrs_cli::{AgentCommands, Cli};
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;
use wg_quickrs::{WG_QUICKRS_CONFIG_FOLDER, WG_QUICKRS_CONFIG_FILE};
use wg_quickrs_wasm::validation::error::ValidationError;
use wg_quickrs_wasm::macros::full_version;
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
        wg_quickrs_cli::Commands::Init(init_opts) => commands::init::initialize_agent(init_opts)?,
        wg_quickrs_cli::Commands::Agent {
            commands,
        } => match commands {
            // wg-quickrs agent run
            AgentCommands::Run => commands::agent::run_agent().await?,
            // wg-quickrs agent set-web-address
            AgentCommands::SetWebAddress(v) => commands::config::set_agent_web_address(&v.address)?,
            // wg-quickrs agent enable-web-http
            AgentCommands::EnableWebHttp => commands::config::toggle_agent_web_http(true)?,
            // wg-quickrs agent disable-web-http
            AgentCommands::DisableWebHttp => commands::config::toggle_agent_web_http(false)?,
            // wg-quickrs agent set-http-web-port
            AgentCommands::SetWebHttpPort(v) => commands::config::set_agent_web_http_port(v.port)?,
            // wg-quickrs agent enable-web-https
            AgentCommands::EnableWebHttps => commands::config::toggle_agent_web_https(true)?,
            // wg-quickrs agent disable-web-https
            AgentCommands::DisableWebHttps => commands::config::toggle_agent_web_https(false)?,
            // wg-quickrs agent set-web-https-port
            AgentCommands::SetWebHttpsPort(v) => commands::config::set_agent_web_https_port(v.port)?,
            // wg-quickrs agent set-web-https-tls-cert
            AgentCommands::SetWebHttpsTlsCert(v) => commands::config::set_agent_web_http_tls_cert(&v.path)?,
            // wg-quickrs agent set-web-https-tls-key
            AgentCommands::SetWebHttpsTlsKey(v) => commands::config::set_agent_web_http_tls_key(&v.path)?,
            // wg-quickrs agent enable-web-password
            AgentCommands::EnableWebPassword => commands::config::toggle_agent_web_password(true)?,
            // wg-quickrs agent disable-web-password
            AgentCommands::DisableWebPassword => commands::config::toggle_agent_web_password(false)?,
            // wg-quickrs agent reset-web-password
            AgentCommands::ResetWebPassword(reset_web_password_opts) => commands::config::reset_web_password(reset_web_password_opts)?,
            // wg-quickrs agent enable-vpn
            AgentCommands::EnableVpn => commands::config::toggle_agent_vpn(true)?,
            // wg-quickrs agent disable-vpn
            AgentCommands::DisableVpn => commands::config::toggle_agent_vpn(false)?,
            // wg-quickrs agent set-vpn-port
            AgentCommands::SetVpnPort(v) => commands::config::set_agent_vpn_port(v.port)?,
            // wg-quickrs agent enable-firewall
            AgentCommands::EnableFirewall => commands::config::toggle_agent_firewall(true)?,
            // wg-quickrs agent disable-firewall
            AgentCommands::DisableFirewall => commands::config::toggle_agent_firewall(false)?,
            // wg-quickrs agent set-firewall-utility
            AgentCommands::SetFirewallUtility(v) => commands::config::set_agent_firewall_utility(&v.utility)?,
            // wg-quickrs agent set-firewall-gateway
            AgentCommands::SetFirewallGateway(v) => commands::config::set_agent_firewall_gateway(&v.interface)?,
        },
    };

    Ok(())
}

