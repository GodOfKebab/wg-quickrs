use crate::cli::AgentCommands;
use crate::web::server;
use crate::{WIREGUARD_CONFIG_FILE, conf, wireguard};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

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
        // start the HTTP server with TLS for server and API control
        server::run_http_server(&config, tls_cert, tls_key)
            .await
            .expect("HTTP server failed to start");
    }
    ExitCode::SUCCESS
}
