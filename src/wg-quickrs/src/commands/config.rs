use crate::commands::helpers;
use crate::commands::validation::{check_field_str_agent, check_field_path_agent};
use crate::conf;
use argon2::PasswordHash;
use wg_quickrs_cli::ResetWebPasswordOptions;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

pub fn reset_web_password(reset_web_password_opts: &ResetWebPasswordOptions) -> ExitCode {
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

pub fn toggle_agent_fields(field: &str, status: bool) -> ExitCode {
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
                let mut ret  = check_field_path_agent("path", &config.agent.web.https.tls_cert);
                if !ret.status {
                    log::error!("{}", ret.msg);
                    return ExitCode::FAILURE;
                }
                ret  = check_field_path_agent("path", &config.agent.web.https.tls_key);
                if !ret.status {
                    log::error!("{}", ret.msg);
                    return ExitCode::FAILURE;
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
            config.agent.vpn.enabled = status;
        }
        "firewall" => {
            log::info!(
                "{} firewall setting up NAT and input rules (utility={})...",
                if status { "Enabling" } else { "Disabling" },
                config.agent.firewall.utility.display()
            );

            let ret_utility  = check_field_path_agent("firewall-utility", &config.agent.firewall.utility);
            if status && !ret_utility.status {
                log::error!("{}", ret_utility.msg);
                return ExitCode::FAILURE;
            }
            let ret_gateway  = check_field_str_agent("firewall-gateway", &config.agent.firewall.gateway);
            if status && !ret_gateway.status {
                log::error!("{}", ret_gateway.msg);
                return ExitCode::FAILURE;
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

pub enum AgentFieldValue {
    Text(String),
    Port(u16),
    Path(PathBuf),
}

pub fn set_agent_fields(field: &str, value: AgentFieldValue) -> ExitCode {
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
            let ret  = check_field_str_agent("address", &config.agent.web.address);
            if !ret.status {
                log::error!("{}", ret.msg);
                return ExitCode::FAILURE;
            }
        }
        ("http-port", AgentFieldValue::Port(port)) => {
            config.agent.web.http.port = port;
            log::info!("Setting HTTP port to {}", config.agent.web.http.port);
        }
        ("https-port", AgentFieldValue::Port(port)) => {
            config.agent.web.https.port = port;
            log::info!("Setting HTTPS port to {}", config.agent.web.https.port);
        }
        ("https-tls-cert", AgentFieldValue::Path(cert)) => {
            config.agent.web.https.tls_cert = cert;
            log::info!("Setting TLS certificate to {}", config.agent.web.https.tls_cert.display());
            let ret  = check_field_path_agent("path", &config.agent.web.https.tls_cert);
            if !ret.status {
                log::error!("{}", ret.msg);
                return ExitCode::FAILURE;
            }
        }
        ("https-tls-key", AgentFieldValue::Path(key)) => {
            config.agent.web.https.tls_key = key;
            log::info!("Setting TLS key to {}", config.agent.web.https.tls_key.display());
            let ret  = check_field_path_agent("path", &config.agent.web.https.tls_key);
            if !ret.status {
                log::error!("{}", ret.msg);
                return ExitCode::FAILURE;
            }
        }
        ("vpn-port", AgentFieldValue::Port(port)) => {
            config.agent.vpn.port = port;
            log::info!("Setting VPN port to {}", config.agent.vpn.port);
        }
        ("firewall-utility", AgentFieldValue::Path(value)) => {
            config.agent.firewall.utility = value;
            log::info!("Setting firewall utility to {}", config.agent.firewall.utility.display());
            let ret  = check_field_path_agent("firewall-utility", &config.agent.firewall.utility);
            if !ret.status {
                log::error!("{}", ret.msg);
                return ExitCode::FAILURE;
            }
        }
        ("firewall-gateway", AgentFieldValue::Text(value)) => {
            config.agent.firewall.gateway = value;
            log::info!("Setting firewall gateway to {}",config.agent.firewall.gateway);
            let ret  = check_field_str_agent("firewall-gateway", &config.agent.firewall.gateway);
            if !ret.status {
                log::error!("{}", ret.msg);
                return ExitCode::FAILURE;
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
