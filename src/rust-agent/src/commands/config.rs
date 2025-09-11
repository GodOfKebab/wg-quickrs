use crate::commands::helpers;
use crate::commands::validation::check_field_agent;
use crate::conf;
use argon2::PasswordHash;
use rust_cli::ResetWebPasswordOptions;
use rust_wasm::types::EnabledValue;
use rust_wasm::validation::FieldValue;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

pub(crate) fn reset_web_password(reset_web_password_opts: &ResetWebPasswordOptions) -> ExitCode {
    // get the wireguard config a file path
    let mut config = match conf::util::get_config() {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            return ExitCode::FAILURE;
        }
    };

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
            match io::stdin().read_line(&mut pwd) {
                Ok(_) => pwd,
                Err(e) => {
                    log::error!("Failed to read input: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
    };
    let password_hash = match helpers::calculate_password_hash(password.trim()) {
        // Remove newline character
        Ok(p) => p,
        Err(e) => {
            return e;
        }
    };

    config.agent.web.password.enabled = true;
    config.agent.web.password.hash = password_hash;
    conf::util::set_config(&mut config).expect("Failed to set config");

    ExitCode::SUCCESS
}

fn validate_agent_field(field_name: &str, value: impl ToString) -> Option<ExitCode> {
    let fv = FieldValue {
        str: value.to_string(),
        enabled_value: EnabledValue {
            enabled: true,
            value: value.to_string(),
        },
    };

    let ret = check_field_agent(field_name, &fv);
    if !ret.status {
        log::error!("{}", ret.msg);
        Some(ExitCode::FAILURE)
    } else {
        None
    }
}

pub(crate) fn toggle_agent_fields(field: &str, status: bool) -> ExitCode {
    // get the wireguard config a file path
    let mut config = match conf::util::get_config() {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            return ExitCode::FAILURE;
        }
    };
    match field {
        "http" => {
            log::info!(
                "{} HTTP web server (port={})...",
                if status { "Enabling" } else { "Disabling" },
                config.agent.web.http.port
            );
            if status && let Some(code) = validate_agent_field("port", config.agent.web.http.port) {
                return code;
            }
            config.agent.web.http.enabled = status;
        }
        "https" => {
            log::info!(
                "{} HTTPS web server (port={}, tls_cert={}, tls_key={})...",
                if status { "Enabling" } else { "Disabling" },
                config.agent.web.https.port,
                config.agent.web.https.tls_cert.display(),
                config.agent.web.https.tls_key.display()
            );
            if status {
                if let Some(code) = validate_agent_field("port", config.agent.web.https.port) {
                    return code;
                }
                if let Some(code) =
                    validate_agent_field("path", config.agent.web.https.tls_cert.display())
                {
                    return code;
                }
                if let Some(code) =
                    validate_agent_field("path", config.agent.web.https.tls_key.display())
                {
                    return code;
                }
            }
            config.agent.web.https.enabled = status;
        }
        "password" => {
            log::info!(
                "{} password for the web server...",
                if status { "Enabling" } else { "Disabling" }
            );
            if status {
                match PasswordHash::new(&config.agent.web.password.hash) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("{e}");
                        return ExitCode::FAILURE;
                    }
                }
            }
            config.agent.web.password.enabled = status;
        }
        "vpn" => {
            log::info!(
                "{} VPN server (port={})...",
                if status { "Enabling" } else { "Disabling" },
                config.agent.vpn.port
            );

            if status && let Some(code) = validate_agent_field("port", config.agent.vpn.port) {
                return code;
            }
            config.agent.vpn.enabled = status;
        }
        "firewall" => {
            log::info!(
                "{} firewall setting up NAT and input rules (utility={})...",
                if status { "Enabling" } else { "Disabling" },
                config.agent.firewall.utility.display()
            );

            if status
                && let Some(code) =
                    validate_agent_field("firewall", config.agent.firewall.utility.display())
            {
                return code;
            }
            config.agent.firewall.enabled = status;
        }
        _ => {
            return ExitCode::FAILURE;
        }
    }
    conf::util::set_config(&mut config).expect("Failed to set config");
    log::info!("Done.");

    ExitCode::SUCCESS
}

pub(crate) enum AgentFieldValue {
    Text(String),
    Port(u16),
    Path(PathBuf),
}

pub(crate) fn set_agent_fields(field: &str, value: AgentFieldValue) -> ExitCode {
    // get the wireguard config a file path
    let mut config = match conf::util::get_config() {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            return ExitCode::FAILURE;
        }
    };
    match (field, value) {
        ("address", AgentFieldValue::Text(addr)) => {
            config.agent.web.address = addr;
            log::info!("Setting agent address to {}", config.agent.web.address);
            if let Some(code) = validate_agent_field("address", &config.agent.web.address) {
                return code;
            }
        }
        ("http-port", AgentFieldValue::Port(port)) => {
            config.agent.web.http.port = port;
            log::info!("Setting HTTP port to {}", config.agent.web.http.port);
            if let Some(code) = validate_agent_field("port", config.agent.web.http.port) {
                return code;
            }
        }
        ("https-port", AgentFieldValue::Port(port)) => {
            config.agent.web.https.port = port;
            log::info!("Setting HTTPS port to {}", config.agent.web.https.port);
            if let Some(code) = validate_agent_field("port", config.agent.web.https.port) {
                return code;
            }
        }
        ("https-tls-cert", AgentFieldValue::Path(cert)) => {
            config.agent.web.https.tls_cert = cert;
            log::info!(
                "Setting TLS certificate to {}",
                config.agent.web.https.tls_cert.display()
            );
            if let Some(code) =
                validate_agent_field("path", config.agent.web.https.tls_cert.display())
            {
                return code;
            }
        }
        ("https-tls-key", AgentFieldValue::Path(key)) => {
            config.agent.web.https.tls_key = key;
            log::info!(
                "Setting TLS key to {}",
                config.agent.web.https.tls_key.display()
            );
            if let Some(code) =
                validate_agent_field("path", config.agent.web.https.tls_key.display())
            {
                return code;
            }
        }
        ("vpn-port", AgentFieldValue::Port(port)) => {
            config.agent.vpn.port = port;
            log::info!("Setting VPN port to {}", config.agent.vpn.port);
            if let Some(code) = validate_agent_field("port", config.agent.vpn.port) {
                return code;
            }
        }
        ("firewall-utility", AgentFieldValue::Path(value)) => {
            config.agent.firewall.utility = value;
            log::info!(
                "Setting firewall utility to {}",
                config.agent.firewall.utility.display()
            );
            if let Some(code) =
                validate_agent_field("firewall", config.agent.firewall.utility.display())
            {
                return code;
            }
        }
        _ => {
            return ExitCode::FAILURE;
        }
    }

    conf::util::set_config(&mut config).expect("Failed to set config");
    log::info!("Done.");

    ExitCode::SUCCESS
}
