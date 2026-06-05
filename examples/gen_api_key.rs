//! Helper: generate a random API key + matching SQL inserts for manual testing.
//!
//! Run with: `cargo run --example gen_api_key`
//!
//! Prints the full key (save it — only shown once) and SQL to register it
//! in the `workspaces` and `api_keys` tables.

use argon2::Argon2;
use argon2::password_hash::rand_core::{OsRng, RngCore};
use argon2::password_hash::{PasswordHasher, SaltString};
use uuid::Uuid;

fn main() {
    let mut bytes = [0u8; 24];
    let mut rng = OsRng;
    rng.fill_bytes(&mut bytes);
    let secret = hex::encode(bytes);
    let key = format!("cx_live_{}", secret);
    let prefix = &key[..12];

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(key.as_bytes(), &salt)
        .expect("hash")
        .to_string();

    let workspace_id = Uuid::new_v4();
    let api_key_id = Uuid::new_v4();

    println!("# API key (save this — only shown once):");
    println!("{}\n", key);
    println!("# Run this SQL to register it:");
    println!(
        "INSERT INTO workspaces (id, name) VALUES ('{}', 'default');",
        workspace_id
    );
    println!(
        "INSERT INTO api_keys (id, workspace_id, name, key_prefix, key_hash)\n  VALUES ('{}', '{}', 'default', '{}', '{}');",
        api_key_id, workspace_id, prefix, hash
    );
}
