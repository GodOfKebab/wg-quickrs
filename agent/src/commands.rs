use crate::cli::AgentCommands;
use crate::conf::util::ConfUtilError;
use crate::{WIREGUARD_CONFIG_FILE, conf, server, wireguard};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use rand::{RngCore, rng};
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub(crate) fn initialize_agent() -> ExitCode {
    if let Err(ConfUtilError::FileRead(_, _)) = conf::util::get_config() {
    } else {
        log::error!("wg-rusteze agent is already initialized.");
        return ExitCode::FAILURE;
    }
    log::info!("Initializing wg-rusteze agent...");
    ExitCode::SUCCESS
}

pub(crate) fn reset_web_password() -> ExitCode {
    // get the wireguard config file path
    let mut config = match conf::util::get_config() {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    log::info!("Resetting the web password...");
    print!("Enter your new password: ");
    io::stdout().flush().unwrap(); // Ensure the prompt is shown before waiting for input

    let mut password = String::new();
    match io::stdin().read_line(&mut password) {
        Ok(_) => {}
        Err(e) => {
            log::error!("Failed to read input: {e}");
            return ExitCode::FAILURE;
        }
    }
    let password = password.trim(); // Remove newline character

    let mut sbytes = [0; 8];
    rng().fill_bytes(&mut sbytes);
    let salt = match SaltString::encode_b64(&sbytes) {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(password.as_ref(), &salt) {
        Ok(password_hash) => password_hash.to_string(),
        Err(e) => {
            log::error!("Password hashing failed: {e}");
            return ExitCode::FAILURE;
        }
    };

    config.agent.web.password.enabled = true;
    config.agent.web.password.hash = password_hash;
    conf::util::set_config(&mut config).expect("Failed to set config");

    ExitCode::SUCCESS
}

pub(crate) async fn run_agent(
    wireguard_config_folder: &Path,
    tls_cert: &PathBuf,
    tls_key: &PathBuf,
    commands: &AgentCommands,
) -> ExitCode {
    // get the wireguard config file path
    let config = match conf::util::get_config() {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let mut run_wireguard = true;
    let mut run_web = true;
    match commands {
        AgentCommands::Run(opts) => {
            if opts.only_wireguard {
                run_web = false;
                log::info!("--only-wireguard flag detected. Starting only the wireguard server...")
            } else if opts.only_web {
                run_wireguard = false;
                log::info!("--only-web flag detected. Running only the web configuration portal...")
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
            .set(wireguard_config_folder.join(format!("{}.conf", config.network.identifier)))
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
    ExitCode::SUCCESS
}
