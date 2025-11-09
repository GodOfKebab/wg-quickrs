use crate::commands::helpers;
use crate::conf;
use argon2::PasswordHash;
use wg_quickrs_cli::ResetWebPasswordOptions;
use wg_quickrs_lib::validation::agent::*;
use wg_quickrs_lib::validation::error::*;
use std::io;
use std::io::Write;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use thiserror::Error;
use crate::conf::util::ConfUtilError;
use crate::WG_QUICKRS_CONFIG_FOLDER;
use serde::Serialize;

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
    #[error("Failed to serialize to YAML: {0}")]
    YamlSerialization(#[from] serde_yml::Error),
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

// Helper function to print any serializable struct as YAML
fn print_as_yaml<T: Serialize>(value: &T) -> Result<(), ConfigCommandError> {
    let yaml = serde_yml::to_string(value)?;
    print!("{}", yaml);
    Ok(())
}

// Macro to generate get functions for config fields
macro_rules! impl_config_getter {
    // For simple types that implement Display
    ($fn_name:ident, $($field:ident).+) => {
        pub fn $fn_name() -> Result<(), ConfigCommandError> {
            let config = conf::util::get_config()?;
            println!("{}", config.$($field).+);
            Ok(())
        }
    };
    // For PathBuf types that need .display()
    ($fn_name:ident, $($field:ident).+, display) => {
        pub fn $fn_name() -> Result<(), ConfigCommandError> {
            let config = conf::util::get_config()?;
            println!("{}", config.$($field).+.display());
            Ok(())
        }
    };
    // For structs that should be serialized as YAML
    ($fn_name:ident, $($field:ident).+, yaml) => {
        pub fn $fn_name() -> Result<(), ConfigCommandError> {
            let config = conf::util::get_config()?;
            print_as_yaml(&config.$($field).+)
        }
    };
}

// Get functions - generated using macro
// Full struct getters
impl_config_getter!(get_agent, agent, yaml);
impl_config_getter!(get_agent_web, agent.web, yaml);
impl_config_getter!(get_agent_web_http, agent.web.http, yaml);
impl_config_getter!(get_agent_web_https, agent.web.https, yaml);
impl_config_getter!(get_agent_web_password, agent.web.password, yaml);
impl_config_getter!(get_agent_vpn, agent.vpn, yaml);
impl_config_getter!(get_agent_firewall, agent.firewall, yaml);

// Individual field getters
impl_config_getter!(get_agent_web_address, agent.web.address);
impl_config_getter!(get_agent_web_http_enabled, agent.web.http.enabled);
impl_config_getter!(get_agent_web_http_port, agent.web.http.port);
impl_config_getter!(get_agent_web_https_enabled, agent.web.https.enabled);
impl_config_getter!(get_agent_web_https_port, agent.web.https.port);
impl_config_getter!(get_agent_web_https_tls_cert, agent.web.https.tls_cert, display);
impl_config_getter!(get_agent_web_https_tls_key, agent.web.https.tls_key, display);
impl_config_getter!(get_agent_web_password_enabled, agent.web.password.enabled);
impl_config_getter!(get_agent_web_password_hash, agent.web.password.hash);
impl_config_getter!(get_agent_vpn_enabled, agent.vpn.enabled);
impl_config_getter!(get_agent_vpn_port, agent.vpn.port);
impl_config_getter!(get_agent_firewall_enabled, agent.firewall.enabled);
impl_config_getter!(get_agent_firewall_utility, agent.firewall.utility, display);
impl_config_getter!(get_agent_firewall_gateway, agent.firewall.gateway);

// Command handler - dispatches config commands to appropriate functions
pub fn handle_config_command(target: &wg_quickrs_cli::ConfigCommands) -> Result<(), ConfigCommandError> {
    use wg_quickrs_cli::*;

    match target {
        ConfigCommands::Enable { target } => match target {
            EnableCommands::Agent { target } => match target {
                EnableAgentCommands::Web { target } => match target {
                    EnableAgentWebCommands::Http => toggle_agent_web_http(true),
                    EnableAgentWebCommands::Https => toggle_agent_web_https(true),
                    EnableAgentWebCommands::Password => toggle_agent_web_password(true),
                },
                EnableAgentCommands::Vpn => toggle_agent_vpn(true),
                EnableAgentCommands::Firewall => toggle_agent_firewall(true),
            },
        },
        ConfigCommands::Disable { target } => match target {
            DisableCommands::Agent { target } => match target {
                DisableAgentCommands::Web { target } => match target {
                    DisableAgentWebCommands::Http => toggle_agent_web_http(false),
                    DisableAgentWebCommands::Https => toggle_agent_web_https(false),
                    DisableAgentWebCommands::Password => toggle_agent_web_password(false),
                },
                DisableAgentCommands::Vpn => toggle_agent_vpn(false),
                DisableAgentCommands::Firewall => toggle_agent_firewall(false),
            },
        },
        ConfigCommands::Set { target } => match target {
            SetCommands::Agent { target } => match target {
                SetAgentCommands::Web { target } => match target {
                    SetAgentWebCommands::Address { value } => set_agent_web_address(value),
                    SetAgentWebCommands::Http { target } => match target {
                        SetAgentWebHttpCommands::Port { value } => set_agent_web_http_port(*value),
                    },
                    SetAgentWebCommands::Https { target } => match target {
                        SetAgentWebHttpsCommands::Port { value } => set_agent_web_https_port(*value),
                        SetAgentWebHttpsCommands::TlsCert { value } => set_agent_web_http_tls_cert(value),
                        SetAgentWebHttpsCommands::TlsKey { value } => set_agent_web_http_tls_key(value),
                    },
                },
                SetAgentCommands::Vpn { target } => match target {
                    SetAgentVpnCommands::Port { value } => set_agent_vpn_port(*value),
                },
                SetAgentCommands::Firewall { target } => match target {
                    SetAgentFirewallCommands::Utility { value } => set_agent_firewall_utility(value),
                    SetAgentFirewallCommands::Gateway { value } => set_agent_firewall_gateway(value),
                },
            },
        },
        ConfigCommands::Reset { target } => match target {
            ResetCommands::Agent { target } => match target {
                ResetAgentCommands::Web { target } => match target {
                    ResetAgentWebCommands::Password { password } => {
                        reset_web_password(&ResetWebPasswordOptions {
                            password: password.clone(),
                        })
                    },
                },
            },
        },
        ConfigCommands::Get { target } => match target {
            GetCommands::Agent { target } => match target {
                None => get_agent(),
                Some(agent_cmd) => match agent_cmd {
                    GetAgentCommands::Web { target } => match target {
                        None => get_agent_web(),
                        Some(web_cmd) => match web_cmd {
                            GetAgentWebCommands::Address => get_agent_web_address(),
                            GetAgentWebCommands::Http { target } => match target {
                                None => get_agent_web_http(),
                                Some(http_cmd) => match http_cmd {
                                    GetAgentWebHttpCommands::Enabled => get_agent_web_http_enabled(),
                                    GetAgentWebHttpCommands::Port => get_agent_web_http_port(),
                                },
                            },
                            GetAgentWebCommands::Https { target } => match target {
                                None => get_agent_web_https(),
                                Some(https_cmd) => match https_cmd {
                                    GetAgentWebHttpsCommands::Enabled => get_agent_web_https_enabled(),
                                    GetAgentWebHttpsCommands::Port => get_agent_web_https_port(),
                                    GetAgentWebHttpsCommands::TlsCert => get_agent_web_https_tls_cert(),
                                    GetAgentWebHttpsCommands::TlsKey => get_agent_web_https_tls_key(),
                                },
                            },
                            GetAgentWebCommands::Password { target } => match target {
                                None => get_agent_web_password(),
                                Some(pwd_cmd) => match pwd_cmd {
                                    GetAgentWebPasswordCommands::Enabled => get_agent_web_password_enabled(),
                                    GetAgentWebPasswordCommands::Hash => get_agent_web_password_hash(),
                                },
                            },
                        },
                    },
                    GetAgentCommands::Vpn { target } => match target {
                        None => get_agent_vpn(),
                        Some(vpn_cmd) => match vpn_cmd {
                            GetAgentVpnCommands::Enabled => get_agent_vpn_enabled(),
                            GetAgentVpnCommands::Port => get_agent_vpn_port(),
                        },
                    },
                    GetAgentCommands::Firewall { target } => match target {
                        None => get_agent_firewall(),
                        Some(fw_cmd) => match fw_cmd {
                            GetAgentFirewallCommands::Enabled => get_agent_firewall_enabled(),
                            GetAgentFirewallCommands::Utility => get_agent_firewall_utility(),
                            GetAgentFirewallCommands::Gateway => get_agent_firewall_gateway(),
                        },
                    },
                },
            },
        },
    }
}

