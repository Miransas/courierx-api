use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;

use crate::auth::jwt::{Claims, verify_token};
use crate::config::Config;
use crate::error::AppError;

/// Axum extractor that authenticates a request via a `Bearer <jwt>` token.
pub struct AuthenticatedUser(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    Config: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let config = Config::from_ref(state);

        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("missing auth header".into()))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("invalid auth header".into()))?
            .trim();

        let claims = verify_token(token, &config.jwt_secret)
            .map_err(|_| AppError::Unauthorized("invalid or expired token".into()))?;

        Ok(AuthenticatedUser(claims))
    }
}
