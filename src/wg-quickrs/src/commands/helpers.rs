use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use rand::{rng, RngCore};

pub(crate) fn calculate_password_hash(password: &str) -> Result<String, argon2::password_hash::Error> {
    let mut sbytes = [0; 8];
    rng().fill_bytes(&mut sbytes);
    let salt = SaltString::encode_b64(&sbytes)?;

    let argon2 = Argon2::default();
    Ok(argon2.hash_password(password.as_ref(), &salt)?.to_string())
}
