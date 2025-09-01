use crate::cli::ResetWebPasswordOptions;
use crate::commands::helpers;
use crate::conf;
use argon2::PasswordHash;
use std::io;
use std::io::Write;
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

pub(crate) fn toggle_web_password(status: bool) -> ExitCode {
    // get the wireguard config a file path
    let mut config = match conf::util::get_config() {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            return ExitCode::FAILURE;
        }
    };
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
    conf::util::set_config(&mut config).expect("Failed to set config");

    log::info!(
        "{} the web password...",
        if status { "Enabled" } else { "Disabled" }
    );

    ExitCode::SUCCESS
}
