use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use rand::{RngCore, rng};
use std::process::ExitCode;

pub(crate) fn calculate_password_hash(password: &str) -> Result<String, ExitCode> {
    let mut sbytes = [0; 8];
    rng().fill_bytes(&mut sbytes);
    let salt = match SaltString::encode_b64(&sbytes) {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            return Err(ExitCode::FAILURE);
        }
    };

    let argon2 = Argon2::default();
    match argon2.hash_password(password.as_ref(), &salt) {
        Ok(password_hash) => Ok(password_hash.to_string()),
        Err(e) => {
            log::error!("Password hashing failed: {e}");
            Err(ExitCode::FAILURE)
        }
    }
}
