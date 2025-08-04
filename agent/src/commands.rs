use crate::conf;
use crate::conf::util::ConfUtilError;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use rand::{rng, RngCore};
use std::io;
use std::io::Write;

pub(crate) fn initialize_agent() {
    match conf::util::get_config() {
        Err(ConfUtilError::FileRead(_)) => {} // expected case if wg-rusteze config exists; do nothing
        _ => {
            log::error!("wg-rusteze agent is already initialized.");
            return;
        }
    }
    log::info!("Initializing wg-rusteze agent...");
}

pub(crate) fn reset_web_password() {
    // get the wireguard config file path
    let mut config = match conf::util::get_config() {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            return;
        }
    };

    log::info!("Resetting the web password...");
    print!("Enter your new password: ");
    io::stdout().flush().unwrap(); // Ensure the prompt is shown before waiting for input

    let mut password = String::new();
    io::stdin()
        .read_line(&mut password)
        .expect("Failed to read input");
    let password = password.trim(); // Remove newline character

    let mut sbytes = [0; 8];
    rng().fill_bytes(&mut sbytes);
    let salt = SaltString::encode_b64(&sbytes).unwrap();

    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_ref(), &salt)
        .expect("Password hashing failed")
        .to_string();

    config.agent.web.password.enabled = true;
    config.agent.web.password.hash = password_hash;
    conf::util::set_config(&mut config).expect("Failed to set config");
}
