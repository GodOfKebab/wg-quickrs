use crate::commands::helpers;
use crate::conf;
use argon2::PasswordHash;
use rust_cli::ResetWebPasswordOptions;
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
            config.agent.web.http.enabled = status;
            log::info!(
                "{} HTTP web server (port={})...",
                if status { "Enabling" } else { "Disabling" },
                config.agent.web.http.port
            );
        }
        "https" => {
            config.agent.web.https.enabled = status;
            log::info!(
                "{} HTTPS web server (port={}, tls_cert={}, tls_key={})...",
                if status { "Enabling" } else { "Disabling" },
                config.agent.web.https.port,
                config.agent.web.https.tls_cert.display(),
                config.agent.web.https.tls_key.display()
            );
        }
        "vpn" => {
            config.agent.vpn.enabled = status;
            log::info!(
                "{} VPN server (port={})...",
                if status { "Enabling" } else { "Disabling" },
                config.agent.vpn.port
            );
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
            config.agent.address = addr;
            log::info!("Setting agent address to {}", config.agent.address);
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
            log::info!(
                "Setting TLS certificate to {}",
                config.agent.web.https.tls_cert.display()
            );
        }
        ("https-tls-key", AgentFieldValue::Path(key)) => {
            config.agent.web.https.tls_key = key;
            log::info!(
                "Setting TLS key to {}",
                config.agent.web.https.tls_key.display()
            );
        }
        ("vpn-port", AgentFieldValue::Port(port)) => {
            config.agent.vpn.port = port;
            log::info!("Setting VPN port to {}", config.agent.vpn.port);
        }
        _ => {
            return ExitCode::FAILURE;
        }
    }

    conf::util::set_config(&mut config).expect("Failed to set config");
    log::info!("Done.");

    ExitCode::SUCCESS
}
