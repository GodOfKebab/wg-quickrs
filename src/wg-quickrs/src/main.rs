use crate::commands::config::AgentFieldValue;
use log::LevelFilter;
use wg_quickrs_cli::AgentCommands;
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use std::process::ExitCode;
use wg_quickrs::{WG_QUICKRS_CONFIG_FOLDER, WIREGUARD_CONFIG_FILE, WG_QUICKRS_CONFIG_FILE};

mod cli;
mod commands;
mod conf;
mod macros;
mod web;
mod wireguard;


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

    // get the wg_quickrs config file path
    let config_folder = expand_tilde(args.wg_quickrs_config_folder.clone())
        .canonicalize()
        .expect("Failed to set WG_QUICKRS_CONFIG_FOLDER");
    WG_QUICKRS_CONFIG_FOLDER
        .set(config_folder.clone())
        .expect("Failed to set WG_QUICKRS_CONFIG_FOLDER");
    let mut wg_quickrs_config_file = config_folder;
    wg_quickrs_config_file.push("conf.yml");
    WG_QUICKRS_CONFIG_FILE
        .set(wg_quickrs_config_file)
        .expect("Failed to set WG_QUICKRS_CONFIG_FILE");
    log::info!(
        "using the wg-quickrs config file at \"{}\"",
        WG_QUICKRS_CONFIG_FILE.get().unwrap().display()
    );

    match &args.command {
        wg_quickrs_cli::Commands::Init(init_opts) => commands::init::initialize_agent(init_opts),
        wg_quickrs_cli::Commands::Agent {
            wireguard_config_folder,
            commands,
        } => match commands {
            // wg-quickrs agent run
            AgentCommands::Run => commands::agent::run_agent(wireguard_config_folder).await,
            // wg-quickrs agent set-web-address
            AgentCommands::SetWebAddress(v) => commands::config::set_agent_fields(
                "address",
                AgentFieldValue::Text(v.address.clone()),
            ),
            // wg-quickrs agent enable-web-http
            AgentCommands::EnableWebHttp => commands::config::toggle_agent_fields("http", true),
            // wg-quickrs agent disable-web-http
            AgentCommands::DisableWebHttp => commands::config::toggle_agent_fields("http", false),
            // wg-quickrs agent set-http-web-port
            AgentCommands::SetWebHttpPort(v) => {
                commands::config::set_agent_fields("http-port", AgentFieldValue::Port(v.port))
            }
            // wg-quickrs agent enable-web-https
            AgentCommands::EnableWebHttps => commands::config::toggle_agent_fields("https", true),
            // wg-quickrs agent disable-web-https
            AgentCommands::DisableWebHttps => commands::config::toggle_agent_fields("https", false),
            // wg-quickrs agent set-web-https-port
            AgentCommands::SetWebHttpsPort(v) => {
                commands::config::set_agent_fields("https-port", AgentFieldValue::Port(v.port))
            }
            // wg-quickrs agent set-web-https-tls-cert
            AgentCommands::SetWebHttpsTlsCert(v) => commands::config::set_agent_fields(
                "https-tls-cert",
                AgentFieldValue::Path(v.path.clone()),
            ),
            // wg-quickrs agent set-web-https-tls-key
            AgentCommands::SetWebHttpsTlsKey(v) => commands::config::set_agent_fields(
                "https-tls-key",
                AgentFieldValue::Path(v.path.clone()),
            ),
            // wg-quickrs agent enable-web-password
            AgentCommands::EnableWebPassword => {
                commands::config::toggle_agent_fields("password", true)
            }
            // wg-quickrs agent disable-web-password
            AgentCommands::DisableWebPassword => {
                commands::config::toggle_agent_fields("password", false)
            }
            // wg-quickrs agent reset-web-password
            AgentCommands::ResetWebPassword(reset_web_password_opts) => {
                commands::config::reset_web_password(reset_web_password_opts)
            }
            // wg-quickrs agent enable-vpn
            AgentCommands::EnableVpn => commands::config::toggle_agent_fields("vpn", true),
            // wg-quickrs agent disable-vpn
            AgentCommands::DisableVpn => commands::config::toggle_agent_fields("vpn", false),
            // wg-quickrs agent set-vpn-port
            AgentCommands::SetVpnPort(v) => {
                commands::config::set_agent_fields("vpn-port", AgentFieldValue::Port(v.port))
            }
            // wg-quickrs agent enable-firewall
            AgentCommands::EnableFirewall => {
                commands::config::toggle_agent_fields("firewall", true)
            }
            // wg-quickrs agent disable-firewall
            AgentCommands::DisableFirewall => {
                commands::config::toggle_agent_fields("firewall", false)
            }
            // wg-quickrs agent set-firewall-utility
            AgentCommands::SetFirewallUtility(v) => {
                commands::config::set_agent_fields(
                    "firewall-utility",
                    AgentFieldValue::Path(v.utility.clone()),
                )
            }
            // wg-quickrs agent set-firewall-gateway
            AgentCommands::SetFirewallGateway(v) => {
                commands::config::set_agent_fields(
                    "firewall-gateway",
                    AgentFieldValue::Text(v.interface.clone()),
                )
            }
        },
    }
}
