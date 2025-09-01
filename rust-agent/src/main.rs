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
        cli::Commands::Init(init_opts) => commands::init::initialize_agent(init_opts),
        cli::Commands::Config { commands } => match commands {
            ConfigCommands::ResetWebPassword(reset_web_password_opts) => {
                commands::config::reset_web_password(reset_web_password_opts)
            }
            ConfigCommands::EnableWebPassword => commands::config::toggle_web_password(true),
            ConfigCommands::DisableWebPassword => commands::config::toggle_web_password(false),
        },
        cli::Commands::Agent {
            wireguard_config_folder,
            tls_cert,
            tls_key,
            commands,
        } => commands::agent::run_agent(wireguard_config_folder, tls_cert, tls_key, commands).await,
    }
}
