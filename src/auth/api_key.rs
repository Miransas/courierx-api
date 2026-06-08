use argon2::password_hash::{PasswordHash, PasswordHasher, SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordVerifier};
use rand::Rng;

/// Length of the visible key prefix stored alongside the hash.
/// Must match `KEY_PREFIX_LEN` in `auth::require_api_key`.
pub const KEY_PREFIX_LEN: usize = 12;

/// Generate a fresh API key.
///
/// Format: `cx_live_<32 lowercase alphanumerics>` (40 chars total).
/// Returns `(full_key, prefix, argon2_hash)`. The caller must store only the
/// prefix and hash; the full key is shown to the user exactly once.
pub fn generate_api_key() -> Result<(String, String, String), argon2::password_hash::Error> {
    let suffix: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect::<String>()
        .to_lowercase();

    let full_key = format!("cx_live_{}", suffix);
    let prefix: String = full_key.chars().take(KEY_PREFIX_LEN).collect();

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(full_key.as_bytes(), &salt)?
        .to_string();

    Ok((full_key, prefix, hash))
}

/// Verify a plaintext API key against a stored Argon2 hash.
#[allow(dead_code)]
pub fn verify_api_key(plain_key: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(plain_key.as_bytes(), &parsed)
        .is_ok()
}
