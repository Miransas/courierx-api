use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT claims for an authenticated user session.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// User id.
    pub sub: Uuid,
    /// Primary workspace id at issuance time.
    pub workspace_id: Uuid,
    /// Expiry (unix seconds).
    pub exp: i64,
    /// Issued at (unix seconds).
    pub iat: i64,
}

/// Sign a new session token. Uses HS256 (the `Header::default()` algorithm).
pub fn create_token(
    user_id: Uuid,
    workspace_id: Uuid,
    secret: &str,
    expiry_days: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        workspace_id,
        exp: (now + Duration::days(expiry_days)).timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Verify and decode a session token. Expiry is checked by default.
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}
