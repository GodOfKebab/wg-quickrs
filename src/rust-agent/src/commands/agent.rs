use crate::web::server;
use crate::{conf, wireguard, WIREGUARD_CONFIG_FILE};
use std::path::Path;
use std::process::ExitCode;

pub(crate) async fn run_agent(wireguard_config_folder: &Path) -> ExitCode {
    // get the wireguard config file path
    let config = match conf::util::get_config() {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    // TODO: start and collect telemetry from wg on a separate thread
    if config.agent.vpn.enabled {
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

    // start the web server(s) if enabled
    server::run_web_server(&config)
        .await
        .expect("web server failed to start");

    ExitCode::SUCCESS
}
