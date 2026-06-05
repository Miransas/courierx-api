//! Helper: hash a fixed test API key and emit SQL to register it.
//!
//! Run with: `cargo run --example gen_api_key`
//!
//! The key string and IDs are hardcoded so the test setup is reproducible.
//! Argon2 salt is random per run, so the `key_hash` differs each run — but
//! every hash verifies against the same `API_KEY` because the salt is
//! embedded in the PHC string.
//!
//! Uses `Argon2::default()` — the same constructor `auth::require_api_key`
//! uses for `verify_password`, so parameters match.

use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHasher, SaltString};
use std::fs;
use uuid::Uuid;

const API_KEY: &str = "cx_live_test1234567890abcdef0123456789ab";
const SEED_PATH: &str = "/tmp/courierx_seed.sql";

fn main() {
    let workspace_id = Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0001);
    let api_key_id = Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0002);

    let prefix = &API_KEY[..12];

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(API_KEY.as_bytes(), &salt)
        .expect("hash")
        .to_string();

    let sql = format!(
        "INSERT INTO workspaces (id, name) VALUES ('{ws}', 'default');\n\
         INSERT INTO api_keys (id, workspace_id, name, key_prefix, key_hash)\n  \
         VALUES ('{ak}', '{ws}', 'default', '{prefix}', '{hash}');\n",
        ws = workspace_id,
        ak = api_key_id,
        prefix = prefix,
        hash = hash,
    );

    fs::write(SEED_PATH, &sql).expect("write seed file");

    println!("API key:    {API_KEY}");
    println!("key_prefix: {prefix}");
    println!("key_hash:   {hash}");
    println!();
    println!("Wrote seed SQL to {SEED_PATH}");
    println!("Next: psql -U courierx -d courierx -f {SEED_PATH}");
}
