use crate::conf;
use crate::conf::util::ConfUtilError;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use rand::{rng, RngCore};
use std::io;
use std::io::Write;

pub(crate) fn initialize_agent() -> i32 {
    if let Err(ConfUtilError::FileRead(_, _)) = conf::util::get_config() {} else {
        log::error!("wg-rusteze agent is already initialized.");
        return 1;
    }
    log::info!("Initializing wg-rusteze agent...");

    0  // exit code for success
}

pub(crate) fn reset_web_password() -> i32 {
    // get the wireguard config file path
    let mut config = match conf::util::get_config() {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            return 1;
        }
    };

    log::info!("Resetting the web password...");
    print!("Enter your new password: ");
    io::stdout().flush().unwrap(); // Ensure the prompt is shown before waiting for input

    let mut password = String::new();
    match io::stdin().read_line(&mut password) {
        Ok(_) => {},
        Err(e) => {
            log::error!("Failed to read input: {e}");
            return 1;
        }
    }
    let password = password.trim(); // Remove newline character

    let mut sbytes = [0; 8];
    rng().fill_bytes(&mut sbytes);
    let salt = match SaltString::encode_b64(&sbytes) {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            return 1;
        }
    };

    let argon2 = Argon2::default();
    let password_hash = match argon2
        .hash_password(password.as_ref(), &salt) {
        Ok(password_hash) => password_hash.to_string(),
        Err(e) => {
            log::error!("Password hashing failed: {e}");
            return 1;
        }
    };

    config.agent.web.password.enabled = true;
    config.agent.web.password.hash = password_hash;
    conf::util::set_config(&mut config).expect("Failed to set config");

    0  // exit code for success
}
