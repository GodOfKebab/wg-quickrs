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

// ============================================================================
// Macros for reducing boilerplate
// ============================================================================

/// Macro for implementing toggle (enable/disable) functions
macro_rules! impl_toggle {
    // Without validation
    ($fn_name:ident, $($field:ident).+ => $log_format:expr) => {
        pub fn $fn_name(status: bool) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            log::info!(
                "{} {}",
                if status { "Enabling" } else { "Disabling" },
                $log_format(&config)
            );
            config.$($field).+.enabled = status;
            conf::util::set_config(&mut config)?;
            Ok(())
        }
    };
    // With validation
    ($fn_name:ident, $($field:ident).+ => $log_format:expr, validate: $validator:expr) => {
        pub fn $fn_name(status: bool) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            log::info!(
                "{} {}",
                if status { "Enabling" } else { "Disabling" },
                $log_format(&config)
            );
            if status {
                $validator(&config)?;
            }
            config.$($field).+.enabled = status;
            conf::util::set_config(&mut config)?;
            Ok(())
        }
    };
}

/// Macro for implementing port setter functions
macro_rules! impl_port_setter {
    ($fn_name:ident, $($field:ident).+, $port_name:expr) => {
        pub fn $fn_name(port: u16) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            config.$($field).+.port = port;
            log::info!("Setting {} port to {}", $port_name, port);
            conf::util::set_config(&mut config)?;
            Ok(())
        }
    };
}

/// Macro for implementing generic setter functions
macro_rules! impl_setter {
    // Simple setter with Display
    ($fn_name:ident, $value_type:ty, $($field:ident).+, $field_name:expr) => {
        pub fn $fn_name(value: &$value_type) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            log::info!("Setting {} to {}", $field_name, value);
            config.$($field).+ = value.clone();
            conf::util::set_config(&mut config)?;
            Ok(())
        }
    };
    // Setter with custom display format
    ($fn_name:ident, $value_type:ty, $($field:ident).+, $field_name:expr, display: $display_fn:expr) => {
        pub fn $fn_name(value: &$value_type) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            log::info!("Setting {} to {}", $field_name, $display_fn(value));
            config.$($field).+ = value.clone();
            conf::util::set_config(&mut config)?;
            Ok(())
        }
    };
    // Setter with transformation/validation
    ($fn_name:ident, $value_type:ty, $($field:ident).+, $field_name:expr, transform: $transformer:expr) => {
        pub fn $fn_name(value: &$value_type) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            log::info!("Setting {} to {}", $field_name, value);
            config.$($field).+ = $transformer(value)?;
            conf::util::set_config(&mut config)?;
            Ok(())
        }
    };
    // Setter with transformation and custom display
    ($fn_name:ident, $value_type:ty, $($field:ident).+, $field_name:expr, display: $display_fn:expr, transform: $transformer:expr) => {
        pub fn $fn_name(value: &$value_type) -> Result<(), ConfigCommandError> {
            let mut config = conf::util::get_config()?;
            log::info!("Setting {} to {}", $field_name, $display_fn(value));
            config.$($field).+ = $transformer(value)?;
            conf::util::set_config(&mut config)?;
            Ok(())
        }
    };
}

// ============================================================================
// Agent Web Configuration Functions
// ============================================================================

impl_setter!(set_agent_web_address, Ipv4Addr, agent.web.address, "agent address");

impl_toggle!(
    toggle_agent_web_http,
    agent.web.http =>
    |c: &wg_quickrs_lib::types::config::Config| format!("HTTP web server (port={})...", c.agent.web.http.port)
);

impl_port_setter!(set_agent_web_http_port, agent.web.http, "HTTP");

impl_setter!(
    set_agent_web_http_tls_cert,
    PathBuf,
    agent.web.https.tls_cert,
    "TLS certificate",
    display: |p: &PathBuf| format!("{}", p.display()),
    transform: |tls_cert: &PathBuf| {
        let wg_quickrs_conf_folder = WG_QUICKRS_CONFIG_FOLDER.get().unwrap();
        validate_tls_file(wg_quickrs_conf_folder, tls_cert)
    }
);

impl_setter!(
    set_agent_web_http_tls_key,
    PathBuf,
    agent.web.https.tls_key,
    "TLS key",
    display: |p: &PathBuf| format!("{}", p.display()),
    transform: |tls_key: &PathBuf| {
        let wg_quickrs_conf_folder = WG_QUICKRS_CONFIG_FOLDER.get().unwrap();
        validate_tls_file(wg_quickrs_conf_folder, tls_key)
    }
);

impl_toggle!(
    toggle_agent_web_https,
    agent.web.https =>
    |c: &wg_quickrs_lib::types::config::Config| format!(
        "HTTPS web server (port={}, tls_cert={}, tls_key={})...",
        c.agent.web.https.port,
        c.agent.web.https.tls_cert.display(),
        c.agent.web.https.tls_key.display()
    ),
    validate: |c: &wg_quickrs_lib::types::config::Config| -> Result<(), ConfigCommandError> {
        let wg_quickrs_conf_folder = WG_QUICKRS_CONFIG_FOLDER.get().unwrap();
        validate_tls_file(wg_quickrs_conf_folder, &c.agent.web.https.tls_cert)?;
        validate_tls_file(wg_quickrs_conf_folder, &c.agent.web.https.tls_key)?;
        Ok(())
    }
);

impl_port_setter!(set_agent_web_https_port, agent.web.https, "HTTPS");

impl_toggle!(
    toggle_agent_web_password,
    agent.web.password =>
    |_: &wg_quickrs_lib::types::config::Config| "password for the web server...".to_string(),
    validate: |c: &wg_quickrs_lib::types::config::Config| -> Result<(), ConfigCommandError> {
        PasswordHash::new(&c.agent.web.password.hash)?;
        Ok(())
    }
);

// ============================================================================
// Special Configuration Functions (not macro-generated)
// ============================================================================

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

// ============================================================================
// Agent VPN Configuration Functions
// ============================================================================

impl_toggle!(
    toggle_agent_vpn,
    agent.vpn =>
    |c: &wg_quickrs_lib::types::config::Config| format!("VPN server (port={})...", c.agent.vpn.port)
);

impl_port_setter!(set_agent_vpn_port, agent.vpn, "VPN");

// ============================================================================
// Agent Firewall Configuration Functions
// ============================================================================

impl_toggle!(
    toggle_agent_firewall,
    agent.firewall =>
    |c: &wg_quickrs_lib::types::config::Config| format!(
        "firewall setting up NAT and input rules (utility={})...",
        c.agent.firewall.utility.display()
    ),
    validate: |c: &wg_quickrs_lib::types::config::Config| -> Result<(), ConfigCommandError> {
        if c.agent.firewall.gateway.is_empty() {
            return Err(ConfigCommandError::GatewayNotSet());
        }
        validate_fw_utility(&c.agent.firewall.utility)?;
        Ok(())
    }
);

impl_setter!(
    set_agent_firewall_utility,
    PathBuf,
    agent.firewall.utility,
    "firewall utility",
    display: |p: &PathBuf| format!("{}", p.display()),
    transform: |utility: &PathBuf| validate_fw_utility(utility)
);

impl_setter!(
    set_agent_firewall_gateway,
    str,
    agent.firewall.gateway,
    "firewall gateway",
    transform: |gateway: &str| parse_and_validate_fw_gateway(gateway)
);

// ============================================================================
// Configuration Getter Functions
// ============================================================================

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

