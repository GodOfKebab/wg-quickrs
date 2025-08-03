use crate::cli::{AgentCommands, ConfigCommands};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use clap::Parser;
use log::LevelFilter;
use once_cell::sync::OnceCell;
use rand::{rng, RngCore};
use simple_logger::SimpleLogger;
use std::io;
use std::io::Write;
use std::path::PathBuf;

mod api;
mod app;
mod cli;
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

    match &args.command {
        cli::Commands::Init {} => {
            log::info!("Initializing wg-rusteze agent..."); // TODO: implement me
            Ok(())
        }
        cli::Commands::Config {
            wg_rusteze_config_file,
            commands,
        } => {
            // get the wg_rusteze config file path
            WG_RUSTEZE_CONFIG_FILE
                .set(wg_rusteze_config_file.clone())
                .expect("Failed to set WG_RUSTEZE_CONFIG_FILE");
            log::info!(
                "using the wg-rusteze config file at \"{}\"",
                WG_RUSTEZE_CONFIG_FILE.get().unwrap().display()
            );

            // get the wireguard config file path
            let mut config = conf::util::get_config();

            match commands {
                ConfigCommands::ResetWebPassword => {
                    log::info!("Resetting the web password..."); // TODO: implement me
                    print!("Enter your new password: ");
                    io::stdout().flush()?; // Ensure the prompt is shown before waiting for input

                    let mut password = String::new();
                    io::stdin()
                        .read_line(&mut password)
                        .expect("Failed to read input");
                    let password = password.trim(); // Remove newline character

                    let mut sbytes = [0; 8];
                    rng().fill_bytes(&mut sbytes);
                    let salt = SaltString::encode_b64(&sbytes).unwrap();

                    let argon2 = Argon2::default();
                    let password_hash = argon2
                        .hash_password(password.as_ref(), &salt)
                        .expect("Password hashing failed")
                        .to_string();

                    config.agent.web.password.enabled = true;
                    config.agent.web.password.hash = password_hash;
                    conf::util::set_config(&config);
                }
            }
            Ok(())
        }
        cli::Commands::Agent {
            wg_rusteze_config_file,
            wireguard_config_folder,
            tls_cert,
            tls_key,
            commands,
        } => {
            // get the wg_rusteze config file path
            WG_RUSTEZE_CONFIG_FILE
                .set(wg_rusteze_config_file.clone())
                .expect("Failed to set WG_RUSTEZE_CONFIG_FILE");
            log::info!(
                "using the wg-rusteze config file at \"{}\"",
                WG_RUSTEZE_CONFIG_FILE.get().unwrap().display()
            );

            // get the wireguard config file path
            let config = conf::util::get_config();

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
