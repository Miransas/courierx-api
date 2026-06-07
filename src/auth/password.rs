use argon2::password_hash::Error as PwError;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

/// Hash a password using Argon2id with a random salt.
pub fn hash_password(password: &str) -> Result<String, PwError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(hash)
}

/// Verify a password against a stored Argon2 hash. Returns `Ok(true)` on
/// match, `Ok(false)` on mismatch. Hash-parse errors surface as `Err`.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, PwError> {
    let parsed = PasswordHash::new(hash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}
