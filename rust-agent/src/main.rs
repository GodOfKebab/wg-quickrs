use crate::commands::config::AgentFieldValue;
use log::LevelFilter;
use once_cell::sync::OnceCell;
use rust_cli::AgentCommands;
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use std::process::ExitCode;

mod cli;
mod commands;
mod conf;
mod macros;
mod web;
mod wireguard;

pub static WG_RUSTEZE_CONFIG_FOLDER: OnceCell<PathBuf> = OnceCell::new();
pub static WG_RUSTEZE_CONFIG_FILE: OnceCell<PathBuf> = OnceCell::new();
pub static WIREGUARD_CONFIG_FILE: OnceCell<PathBuf> = OnceCell::new();

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

    // get the wg_rusteze config file path
    let config_folder = expand_tilde(args.wg_rusteze_config_folder.clone());
    WG_RUSTEZE_CONFIG_FOLDER
        .set(config_folder.clone())
        .expect("Failed to set WG_RUSTEZE_CONFIG_FOLDER");
    let mut wg_rusteze_config_file = config_folder;
    wg_rusteze_config_file.push("conf.yml");
    WG_RUSTEZE_CONFIG_FILE
        .set(wg_rusteze_config_file)
        .expect("Failed to set WG_RUSTEZE_CONFIG_FILE");
    log::info!(
        "using the wg-rusteze config file at \"{}\"",
        WG_RUSTEZE_CONFIG_FILE.get().unwrap().display()
    );

    match &args.command {
        rust_cli::Commands::Init(init_opts) => commands::init::initialize_agent(init_opts),
        rust_cli::Commands::Agent {
            wireguard_config_folder,
            commands,
        } => match commands {
            // wg-rusteze agent run
            AgentCommands::Run => commands::agent::run_agent(wireguard_config_folder).await,
            // wg-rusteze agent set-address
            AgentCommands::SetAddress(v) => commands::config::set_agent_fields(
                "address",
                AgentFieldValue::Text(v.address.clone()),
            ),
            // wg-rusteze agent enable-web-http
            AgentCommands::EnableWebHttp => commands::config::toggle_agent_fields("http", true),
            // wg-rusteze agent disable-web-http
            AgentCommands::DisableWebHttp => commands::config::toggle_agent_fields("http", false),
            // wg-rusteze agent set-http-web-port
            AgentCommands::SetHttpWebPort(v) => {
                commands::config::set_agent_fields("http-port", AgentFieldValue::Port(v.port))
            }
            // wg-rusteze agent enable-web-https
            AgentCommands::EnableWebHttps => commands::config::toggle_agent_fields("https", true),
            // wg-rusteze agent disable-web-https
            AgentCommands::DisableWebHttps => commands::config::toggle_agent_fields("https", false),
            // wg-rusteze agent set-web-https-port
            AgentCommands::SetWebHttpsPort(v) => {
                commands::config::set_agent_fields("https-port", AgentFieldValue::Port(v.port))
            }
            // wg-rusteze agent set-web-https-tls-cert
            AgentCommands::SetWebHttpsTlsCert(v) => commands::config::set_agent_fields(
                "https-tls-cert",
                AgentFieldValue::Path(v.path.clone()),
            ),
            // wg-rusteze agent set-web-https-tls-key
            AgentCommands::SetWebHttpsTlsKey(v) => commands::config::set_agent_fields(
                "https-tls-key",
                AgentFieldValue::Path(v.path.clone()),
            ),
            // wg-rusteze agent enable-vpn
            AgentCommands::EnableVpn => commands::config::toggle_agent_fields("vpn", true),
            // wg-rusteze agent disable-vpn
            AgentCommands::DisableVpn => commands::config::toggle_agent_fields("vpn", false),
            // wg-rusteze agent set-vpn-port
            AgentCommands::SetVpnPort(v) => {
                commands::config::set_agent_fields("vpn-port", AgentFieldValue::Port(v.port))
            }
            // wg-rusteze agent enable-web-password
            AgentCommands::EnableWebPassword => {
                commands::config::toggle_agent_fields("password", true)
            }
            // wg-rusteze agent disable-web-password
            AgentCommands::DisableWebPassword => {
                commands::config::toggle_agent_fields("password", false)
            }
            // wg-rusteze agent reset-web-password
            AgentCommands::ResetWebPassword(reset_web_password_opts) => {
                commands::config::reset_web_password(reset_web_password_opts)
            }
        },
    }
}
