use crate::web::server;
use crate::{conf, wireguard};
use thiserror::Error;
use tokio::try_join;
use crate::conf::util::ConfUtilError;

#[derive(Error, Debug)]
pub enum AgentRunError {
    #[error("configuration error: {0}")]
    Conf(#[from] ConfUtilError),
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
}

pub async fn run_agent() -> Result<(), AgentRunError> {
    let config = conf::util::get_config()?;
    let web_future = server::run_web_server(&config);
    let vpn_future = wireguard::cmd::run_vpn_server(&config);
    try_join!(web_future, vpn_future)?;
    Ok(())
}
