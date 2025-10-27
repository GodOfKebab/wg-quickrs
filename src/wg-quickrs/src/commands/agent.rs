use crate::web::server;
use crate::{conf, wireguard};
use std::process::ExitCode;
use tokio::try_join;

pub async fn run_agent() -> ExitCode {
    // get the wireguard config file path
    let config = match conf::util::get_config() {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let web_future = server::run_web_server(&config);
    let vpn_future = wireguard::cmd::run_vpn_server(&config);
    match try_join!(web_future, vpn_future).map(|_| ()) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            log::error!("{e}");
            ExitCode::FAILURE
        }
    }
}
