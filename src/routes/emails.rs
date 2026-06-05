use crate::auth::AuthContext;
use crate::error::AppError;
use axum::Json;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

/// Accept either a single string or an array of strings for `to`/`cc`/`bcc`,
/// matching Resend's request shape.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum StringOrVec {
    Single(String),
    Multiple(Vec<String>),
}

impl StringOrVec {
    fn into_vec(self) -> Vec<String> {
        match self {
            StringOrVec::Single(s) => vec![s],
            StringOrVec::Multiple(v) => v,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateEmail {
    pub from: String,
    pub to: StringOrVec,
    pub cc: Option<StringOrVec>,
    pub bcc: Option<StringOrVec>,
    pub reply_to: Option<String>,
    pub subject: String,
    pub html: Option<String>,
    pub text: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct CreateEmailResponse {
    pub id: Uuid,
    pub status: &'static str,
}

/// `POST /v1/emails` — queue an email for delivery. Returns 202 with the
/// generated id. The actual send is the worker's job (not in this skeleton).
pub async fn create(
    State(pool): State<PgPool>,
    Extension(auth): Extension<AuthContext>,
    Json(body): Json<CreateEmail>,
) -> Result<impl IntoResponse, AppError> {
    if body.from.trim().is_empty() {
        return Err(AppError::Validation("from is required".into()));
    }
    if body.subject.trim().is_empty() {
        return Err(AppError::Validation("subject is required".into()));
    }

    let to_addrs = body.to.into_vec();
    if to_addrs.is_empty() || to_addrs.iter().any(|s| s.trim().is_empty()) {
        return Err(AppError::Validation(
            "to must contain at least one non-empty address".into(),
        ));
    }

    let has_html = body.html.as_deref().is_some_and(|s| !s.is_empty());
    let has_text = body.text.as_deref().is_some_and(|s| !s.is_empty());
    if !has_html && !has_text {
        return Err(AppError::Validation("html or text body is required".into()));
    }

    let cc_addrs = body.cc.map(StringOrVec::into_vec);
    let bcc_addrs = body.bcc.map(StringOrVec::into_vec);
    let headers: Value = serde_json::to_value(body.headers.unwrap_or_default())
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO emails (
            id, workspace_id, api_key_id,
            from_addr, to_addrs, cc_addrs, bcc_addrs, reply_to,
            subject, html_body, text_body, headers, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'queued')
        "#,
    )
    .bind(id)
    .bind(auth.workspace_id)
    .bind(auth.api_key_id)
    .bind(&body.from)
    .bind(&to_addrs)
    .bind(cc_addrs.as_ref())
    .bind(bcc_addrs.as_ref())
    .bind(body.reply_to.as_deref())
    .bind(&body.subject)
    .bind(body.html.as_deref())
    .bind(body.text.as_deref())
    .bind(&headers)
    .execute(&pool)
    .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(CreateEmailResponse {
            id,
            status: "queued",
        }),
    ))
}
