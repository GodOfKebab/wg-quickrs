use crate::cli::ConfigCommands;
use clap::Parser;
use log::LevelFilter;
use once_cell::sync::OnceCell;
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use std::process::ExitCode;

mod cli;
mod commands;
mod conf;
mod macros;
mod web;
mod wireguard;

pub static WG_RUSTEZE_CONFIG_FILE: OnceCell<PathBuf> = OnceCell::new();
pub static WIREGUARD_CONFIG_FILE: OnceCell<PathBuf> = OnceCell::new();

#[actix_web::main]
async fn main() -> ExitCode {
    let args = cli::Cli::parse();
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
    WG_RUSTEZE_CONFIG_FILE
        .set(args.wg_rusteze_config_file.clone())
        .expect("Failed to set WG_RUSTEZE_CONFIG_FILE");
    log::info!(
        "using the wg-rusteze config file at \"{}\"",
        WG_RUSTEZE_CONFIG_FILE.get().unwrap().display()
    );

    match &args.command {
        cli::Commands::Init {
            network_identifier,
            network_subnet,
            agent_peer_name,
            agent_local_address,
            agent_local_web_port,
            agent_local_vpn_port,
            agent_public_address,
            agent_public_vpn_port,
            agent_internal_vpn_address,
            agent_use_tls,
            agent_enable_web_password,
            agent_web_password,
            agent_enable_dns,
            agent_dns_server,
            agent_enable_mtu,
            agent_mtu_value,
            agent_enable_script_pre_up,
            agent_script_pre_up_line,
            agent_enable_script_post_up,
            agent_script_post_up_line,
            agent_enable_script_pre_down,
            agent_script_pre_down_line,
            agent_enable_script_post_down,
            agent_script_post_down_line,
            default_enable_dns,
            default_dns_server,
            default_enable_mtu,
            default_mtu_value,
            default_enable_script_pre_up,
            default_script_pre_up_line,
            default_enable_script_post_up,
            default_script_post_up_line,
            default_enable_script_pre_down,
            default_script_pre_down_line,
            default_enable_script_post_down,
            default_script_post_down_line,
            default_enable_persistent_keepalive,
            default_persistent_keepalive_period,
            no_prompt,
        } => commands::initialize_agent(
            network_identifier.clone(),
            network_subnet.clone(),
            agent_peer_name.clone(),
            agent_local_address.clone(),
            *agent_local_web_port,
            *agent_local_vpn_port,
            agent_public_address.clone(),
            *agent_public_vpn_port,
            agent_internal_vpn_address.clone(),
            *agent_use_tls,
            *agent_enable_web_password,
            agent_web_password.clone(),
            *agent_enable_dns,
            agent_dns_server.clone(),
            *agent_enable_mtu,
            agent_mtu_value.clone(),
            *agent_enable_script_pre_up,
            agent_script_pre_up_line.clone(),
            *agent_enable_script_post_up,
            agent_script_post_up_line.clone(),
            *agent_enable_script_pre_down,
            agent_script_pre_down_line.clone(),
            *agent_enable_script_post_down,
            agent_script_post_down_line.clone(),
            *default_enable_dns,
            default_dns_server.clone(),
            *default_enable_mtu,
            default_mtu_value.clone(),
            *default_enable_script_pre_up,
            default_script_pre_up_line.clone(),
            *default_enable_script_post_up,
            default_script_post_up_line.clone(),
            *default_enable_script_pre_down,
            default_script_pre_down_line.clone(),
            *default_enable_script_post_down,
            default_script_post_down_line.clone(),
            *default_enable_persistent_keepalive,
            default_persistent_keepalive_period.clone(),
            *no_prompt,
        ),
        cli::Commands::Config { commands } => match commands {
            ConfigCommands::ResetWebPassword => commands::reset_web_password(),
        },
        cli::Commands::Agent {
            wireguard_config_folder,
            tls_cert,
            tls_key,
            commands,
        } => commands::run_agent(wireguard_config_folder, tls_cert, tls_key, commands).await,
    }
}
