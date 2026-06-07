use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Router, response::IntoResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use crate::auth::jwt::create_token;
use crate::auth::middleware::AuthenticatedUser;
use crate::auth::password::{hash_password, verify_password};
use crate::config::Config;
use crate::error::AppError;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, message = "password must be at least 8 characters"))]
    pub password: String,
    #[validate(length(min = 1, max = 200))]
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user_id: Uuid,
    pub workspace_id: Uuid,
    pub email: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, message = "password is required"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub user_id: Uuid,
    pub workspace_id: Uuid,
    pub email: String,
    pub name: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/me", get(me))
}

/// `POST /auth/register` — create a user, their default workspace, and an
/// owner membership, returning a fresh JWT.
async fn register(
    State(pool): State<PgPool>,
    State(config): State<Config>,
    Json(body): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let existing: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM users WHERE LOWER(email) = LOWER($1)")
            .bind(&body.email)
            .fetch_optional(&pool)
            .await?;

    if existing.is_some() {
        return Err(AppError::Conflict("email already registered".into()));
    }

    let password_hash = hash_password(&body.password)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("password hash failed: {e}")))?;

    let mut tx = pool.begin().await?;

    let (user_id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, password_hash, name) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(&body.email)
    .bind(&password_hash)
    .bind(&body.name)
    .fetch_one(&mut *tx)
    .await?;

    let workspace_name = body.name.clone().unwrap_or_else(|| "Personal".to_string());
    let workspace_id = Uuid::new_v4();
    sqlx::query("INSERT INTO workspaces (id, name) VALUES ($1, $2)")
        .bind(workspace_id)
        .bind(&workspace_name)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT INTO workspace_members (workspace_id, user_id, role) VALUES ($1, $2, 'owner')",
    )
    .bind(workspace_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let token = create_token(
        user_id,
        workspace_id,
        &config.jwt_secret,
        config.jwt_expiry_days,
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("jwt sign failed: {e}")))?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            user_id,
            workspace_id,
            email: body.email,
            token,
        }),
    ))
}

/// `POST /auth/login` — verify credentials and return a fresh JWT for the
/// user's first-joined workspace.
async fn login(
    State(pool): State<PgPool>,
    State(config): State<Config>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let row: Option<(Uuid, String, String)> =
        sqlx::query_as("SELECT id, email, password_hash FROM users WHERE LOWER(email) = LOWER($1)")
            .bind(&body.email)
            .fetch_optional(&pool)
            .await?;

    let (user_id, email, password_hash) =
        row.ok_or_else(|| AppError::Unauthorized("invalid credentials".into()))?;

    let valid = verify_password(&body.password, &password_hash)
        .map_err(|_| AppError::Unauthorized("invalid credentials".into()))?;

    if !valid {
        return Err(AppError::Unauthorized("invalid credentials".into()));
    }

    let workspace: Option<(Uuid,)> = sqlx::query_as(
        "SELECT workspace_id FROM workspace_members
         WHERE user_id = $1
         ORDER BY joined_at ASC
         LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(&pool)
    .await?;

    let (workspace_id,) =
        workspace.ok_or_else(|| AppError::Unauthorized("no workspace for user".into()))?;

    let token = create_token(
        user_id,
        workspace_id,
        &config.jwt_secret,
        config.jwt_expiry_days,
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("jwt sign failed: {e}")))?;

    Ok(Json(AuthResponse {
        user_id,
        workspace_id,
        email,
        token,
    }))
}

/// `GET /auth/me` — return the current user's basic profile.
async fn me(
    State(pool): State<PgPool>,
    AuthenticatedUser(claims): AuthenticatedUser,
) -> Result<Json<MeResponse>, AppError> {
    let row: Option<(String, Option<String>)> =
        sqlx::query_as("SELECT email, name FROM users WHERE id = $1")
            .bind(claims.sub)
            .fetch_optional(&pool)
            .await?;

    let (email, name) = row.ok_or_else(|| AppError::Unauthorized("user not found".into()))?;

    Ok(Json(MeResponse {
        user_id: claims.sub,
        workspace_id: claims.workspace_id,
        email,
        name,
    }))
}
