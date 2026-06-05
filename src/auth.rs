use crate::error::AppError;
use argon2::password_hash::PasswordHash;
use argon2::{Argon2, PasswordVerifier};
use axum::extract::{Request, State};
use axum::http::header;
use axum::middleware::Next;
use axum::response::Response;
use sqlx::PgPool;
use uuid::Uuid;

/// Auth context attached to authenticated requests via `Extension`.
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub workspace_id: Uuid,
    pub api_key_id: Uuid,
}

const KEY_PREFIX_LEN: usize = 12;

/// Middleware that validates a `Authorization: Bearer <key>` header against
/// the `api_keys` table. On success, inserts `AuthContext` into request
/// extensions and fire-and-forget updates `last_used_at`.
pub async fn require_api_key(
    State(pool): State<PgPool>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let header_val = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::MissingApiKey)?;

    let key = header_val
        .strip_prefix("Bearer ")
        .ok_or(AppError::MissingApiKey)?
        .trim();

    if key.len() < KEY_PREFIX_LEN {
        return Err(AppError::InvalidApiKey);
    }
    let prefix = &key[..KEY_PREFIX_LEN];

    let row: Option<(Uuid, Uuid, String)> =
        sqlx::query_as("SELECT id, workspace_id, key_hash FROM api_keys WHERE key_prefix = $1")
            .bind(prefix)
            .fetch_optional(&pool)
            .await?;

    let (api_key_id, workspace_id, key_hash) = row.ok_or(AppError::InvalidApiKey)?;

    let parsed = PasswordHash::new(&key_hash).map_err(|_| AppError::InvalidApiKey)?;
    Argon2::default()
        .verify_password(key.as_bytes(), &parsed)
        .map_err(|_| AppError::InvalidApiKey)?;

    let pool_clone = pool.clone();
    tokio::spawn(async move {
        let _ = sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(api_key_id)
            .execute(&pool_clone)
            .await;
    });

    req.extensions_mut().insert(AuthContext {
        workspace_id,
        api_key_id,
    });

    Ok(next.run(req).await)
}
