use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, post};
use axum::{Router, response::IntoResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use crate::auth::api_key::generate_api_key;
use crate::auth::middleware::AuthenticatedUser;
use crate::error::AppError;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateApiKeyRequest {
    #[validate(length(min = 1, max = 100, message = "name must be 1-100 characters"))]
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    /// Full plaintext key. Shown only on creation.
    pub key: String,
    pub key_prefix: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ApiKeyListItem {
    pub id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ListApiKeysResponse {
    pub keys: Vec<ApiKeyListItem>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create).get(list))
        .route("/:id", delete(revoke))
}

/// `POST /v1/api-keys` — create a new API key for the caller's workspace.
/// Returns the plaintext key exactly once.
async fn create(
    State(pool): State<PgPool>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Json(body): Json<CreateApiKeyRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let (full_key, prefix, hash) = generate_api_key()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("key generation failed: {e}")))?;

    let id = Uuid::new_v4();
    let (created_at,): (chrono::DateTime<chrono::Utc>,) = sqlx::query_as(
        "INSERT INTO api_keys (id, workspace_id, key_prefix, key_hash, name)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING created_at",
    )
    .bind(id)
    .bind(claims.workspace_id)
    .bind(&prefix)
    .bind(&hash)
    .bind(&body.name)
    .fetch_one(&pool)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateApiKeyResponse {
            id,
            name: body.name,
            key: full_key,
            key_prefix: prefix,
            created_at,
        }),
    ))
}

/// `GET /v1/api-keys` — list keys for the caller's workspace. Hashes are never returned.
async fn list(
    State(pool): State<PgPool>,
    AuthenticatedUser(claims): AuthenticatedUser,
) -> Result<Json<ListApiKeysResponse>, AppError> {
    let keys = sqlx::query_as::<_, ApiKeyListItem>(
        "SELECT id, name, key_prefix, created_at, last_used_at, revoked_at
         FROM api_keys
         WHERE workspace_id = $1
         ORDER BY created_at DESC",
    )
    .bind(claims.workspace_id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(ListApiKeysResponse { keys }))
}

/// `DELETE /v1/api-keys/:id` — mark a key as revoked. Idempotent at the row
/// level but returns 404 if the key is unknown or already revoked.
async fn revoke(
    State(pool): State<PgPool>,
    AuthenticatedUser(claims): AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query(
        "UPDATE api_keys
         SET revoked_at = NOW()
         WHERE id = $1 AND workspace_id = $2 AND revoked_at IS NULL",
    )
    .bind(id)
    .bind(claims.workspace_id)
    .execute(&pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "api key not found or already revoked".into(),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}
