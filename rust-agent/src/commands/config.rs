use crate::commands::helpers;
use crate::conf;
use std::io;
use std::io::Write;
use std::process::ExitCode;

pub(crate) fn reset_web_password() -> ExitCode {
    // get the wireguard config a file path
    let mut config = match conf::util::get_config() {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    log::info!("Resetting the web password...");
    print!("Enter your new password: ");
    io::stdout().flush().unwrap(); // Ensure the prompt is shown before waiting for input

    let mut password = String::new();
    match io::stdin().read_line(&mut password) {
        Ok(_) => {}
        Err(e) => {
            log::error!("Failed to read input: {e}");
            return ExitCode::FAILURE;
        }
    }
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
