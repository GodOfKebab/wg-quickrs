use crate::commands::helpers;
use crate::conf;
use argon2::PasswordHash;
use wg_quickrs_cli::ResetWebPasswordOptions;
use wg_quickrs_wasm::validation::agent::*;
use wg_quickrs_wasm::validation::error::*;
use std::io;
use std::io::Write;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use thiserror::Error;
use crate::conf::util::ConfUtilError;
use crate::WG_QUICKRS_CONFIG_FOLDER;

#[derive(Error, Debug)]
pub enum ConfigCommandError {
    #[error("{0}")]
    PasswordHash(String),
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error(transparent)]
    ConfUtilError(#[from] ConfUtilError),
    #[error("Can't enable firewall gateway: Gateway is not set")]
    GatewayNotSet(),
    #[error("Failed to read input: {0}")]
    ReadFailed(#[from] io::Error),
}

impl From<argon2::password_hash::Error> for ConfigCommandError {
    fn from(err: argon2::password_hash::Error) -> Self {
        ConfigCommandError::PasswordHash(err.to_string())
    }
}

pub fn set_agent_web_address(address: &Ipv4Addr) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    log::info!("Setting agent address to {}", &address);
    config.agent.web.address = address.clone();
    conf::util::set_config(&mut config)?;
    Ok(())
}

pub fn toggle_agent_web_http(status: bool) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    log::info!(
        "{} HTTP web server (port={})...",
        if status { "Enabling" } else { "Disabling" },
        config.agent.web.http.port
    );
    config.agent.web.http.enabled = status;
    conf::util::set_config(&mut config)?;
    Ok(())
}

pub fn set_agent_web_http_port(port: u16) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    config.agent.web.http.port = port;
    log::info!("Setting HTTP port to {}", config.agent.web.http.port);
    conf::util::set_config(&mut config)?;
    Ok(())
}

pub fn set_agent_web_http_tls_cert(tls_cert: &PathBuf) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    log::info!("Setting TLS certificate to {}", &tls_cert.display());
    let wg_quickrs_conf_folder = WG_QUICKRS_CONFIG_FOLDER.get().unwrap();
    config.agent.web.https.tls_cert = validate_tls_file(wg_quickrs_conf_folder, &tls_cert)?;
    conf::util::set_config(&mut config)?;
    Ok(())
}

pub fn set_agent_web_http_tls_key(tls_key: &PathBuf) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    log::info!("Setting TLS key to {}", &tls_key.display());
    let wg_quickrs_conf_folder = WG_QUICKRS_CONFIG_FOLDER.get().unwrap();
    config.agent.web.https.tls_key = validate_tls_file(wg_quickrs_conf_folder, &tls_key)?;
    conf::util::set_config(&mut config)?;
    Ok(())
}

pub fn toggle_agent_web_https(status: bool) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    log::info!(
        "{} HTTPS web server (port={}, tls_cert={}, tls_key={})...",
        if status { "Enabling" } else { "Disabling" },
        config.agent.web.https.port,
        config.agent.web.https.tls_cert.display(),
        config.agent.web.https.tls_key.display()
    );
    if status {
        let wg_quickrs_conf_folder = WG_QUICKRS_CONFIG_FOLDER.get().unwrap();
        let _  = validate_tls_file(wg_quickrs_conf_folder, &config.agent.web.https.tls_cert)?;
        let _  = validate_tls_file(wg_quickrs_conf_folder, &config.agent.web.https.tls_key)?;
    }
    config.agent.web.https.enabled = status;
    conf::util::set_config(&mut config)?;
    Ok(())
}

pub fn set_agent_web_https_port(port: u16) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    config.agent.web.https.port = port;
    log::info!("Setting HTTPS port to {}", config.agent.web.https.port);
    conf::util::set_config(&mut config)?;
    Ok(())
}

pub fn toggle_agent_web_password(status: bool) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    log::info!(
        "{} password for the web server...",
        if status { "Enabling" } else { "Disabling" }
    );
    if status {
        let _ = PasswordHash::new(&config.agent.web.password.hash)?;
    }
    config.agent.web.password.enabled = status;
    conf::util::set_config(&mut config)?;
    Ok(())
}

pub fn reset_web_password(reset_web_password_opts: &ResetWebPasswordOptions) -> Result<(), ConfigCommandError> {
    // get the wireguard config a file path
    let mut config = conf::util::get_config()?;

    log::info!("Resetting the web password...");
    let password = match reset_web_password_opts.password.clone() {
        Some(pwd) => {
            log::warn!(
                "THIS IS HIGHLY INSECURE! Please set the password without the --password flag. The plaintext password could be visible in your shell history."
            );
            pwd
        }
        None => {
            print!("Enter your new password: ");
            io::stdout().flush().unwrap(); // Ensure the prompt is shown before waiting for input

            let mut pwd = String::new();
            io::stdin().read_line(&mut pwd).map_err(|e| ConfigCommandError::ReadFailed(e))?;
            pwd.trim().to_string()
        }
    };
    let password_hash = helpers::calculate_password_hash(password.trim())?;

    config.agent.web.password.hash = password_hash;
    conf::util::set_config(&mut config)?;
    Ok(())
}

pub fn toggle_agent_vpn(status: bool) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    log::info!(
        "{} VPN server (port={})...",
        if status { "Enabling" } else { "Disabling" },
        config.agent.vpn.port
    );
    config.agent.vpn.enabled = status;
    conf::util::set_config(&mut config)?;
    Ok(())
}

pub fn set_agent_vpn_port(port: u16) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    config.agent.vpn.port = port;
    log::info!("Setting VPN port to {}", config.agent.vpn.port);
    conf::util::set_config(&mut config)?;
    Ok(())
}

pub fn toggle_agent_firewall(status: bool) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    log::info!(
        "{} firewall setting up NAT and input rules (utility={})...",
        if status { "Enabling" } else { "Disabling" },
        config.agent.firewall.utility.display()
    );

    if status {
        if config.agent.firewall.gateway.is_empty() {
            return Err(ConfigCommandError::GatewayNotSet())
        }
        let _  = validate_fw_utility(&config.agent.firewall.utility)?;
    }
    config.agent.firewall.enabled = status;
    conf::util::set_config(&mut config)?;
    Ok(())
}

pub fn set_agent_firewall_utility(utility: &PathBuf) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    log::info!("Setting firewall utility to {}", &utility.display());
    config.agent.firewall.utility = validate_fw_utility(&utility)?;
    conf::util::set_config(&mut config)?;
    Ok(())
}

pub fn set_agent_firewall_gateway(gateway: &str) -> Result<(), ConfigCommandError> {
    let mut config = conf::util::get_config()?;
    log::info!("Setting firewall gateway to {}", &gateway);
    config.agent.firewall.gateway = parse_and_validate_fw_gateway(&gateway)?;
    conf::util::set_config(&mut config)?;
    Ok(())
}

