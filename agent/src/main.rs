use crate::cli::{AgentCommands, ConfigCommands};
use clap::Parser;
use log::LevelFilter;
use once_cell::sync::OnceCell;
use simple_logger::SimpleLogger;
use std::io;
use std::io::Error;
use std::path::PathBuf;

mod api;
mod app;
mod cli;
mod commands;
mod conf;
mod macros;
mod server;
mod wireguard;

pub static WG_RUSTEZE_CONFIG_FILE: OnceCell<PathBuf> = OnceCell::new();
pub static WIREGUARD_CONFIG_FILE: OnceCell<PathBuf> = OnceCell::new();

#[actix_web::main]
async fn main() -> io::Result<()> {
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
        cli::Commands::Init {} => {
            let exit_code = commands::initialize_agent();
            std::process::exit(exit_code);
        }
        cli::Commands::Config { commands } => {
            let exit_code = match commands {
                ConfigCommands::ResetWebPassword => commands::reset_web_password(),
            };
            std::process::exit(exit_code);
        }
        cli::Commands::Agent {
            wireguard_config_folder,
            tls_cert,
            tls_key,
            commands,
        } => {
            // get the wireguard config file path
            let config = match conf::util::get_config() {
                Ok(config) => config,
                Err(e) => {
                    log::error!("{e}");
                    return std::process::exit(1);
                }
            };

            let mut run_wireguard = true;
            let mut run_web = true;
            match commands {
                AgentCommands::Run(opts) => {
                    if opts.only_wireguard {
                        run_web = false;
                        log::info!(
                            "--only-wireguard flag detected. Starting only the wireguard server..."
                        )
                    } else if opts.only_web {
                        run_wireguard = false;
                        log::info!(
                            "--only-web flag detected. Running only the web configuration portal..."
                        )
                    } else if opts.all {
                        log::info!(
                            "--all flag detected. Starting the wireguard server and running the web configuration portal..."
                        )
                    } else {
                        log::info!(
                            "No run mode selected. Defaulting to --all (Starting the wireguard server and running the web configuration portal...)"
                        );
                    }
                }
            }

            if run_wireguard {
                WIREGUARD_CONFIG_FILE
                    .set(
                        wireguard_config_folder.join(format!("{}.conf", config.network.identifier)),
                    )
                    .expect("Failed to set WIREGUARD_CONFIG_FILE");
                log::info!(
                    "using the wireguard config file at \"{}\"",
                    WIREGUARD_CONFIG_FILE.get().unwrap().display()
                );

                // start the tunnel
                wireguard::cmd::start_tunnel(&config).unwrap_or_else(|e| {
                    log::error!("{e}");
                });
            }

            if run_web {
                // start the HTTP server with TLS for frontend and API control
                server::run_http_server(&config, tls_cert, tls_key)
                    .await
                    .expect("HTTP server failed to start");
            }
            Ok(())
        }
    }
}
