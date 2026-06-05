mod auth;
mod config;
mod db;
mod error;
mod routes;

use axum::Router;
use axum::middleware;
use axum::routing::{get, post};
use config::Config;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("courierx_api=info,tower_http=info")),
        )
        .init();

    let cfg = Config::from_env()?;
    let pool = db::connect(&cfg.database_url).await?;
    db::migrate(&pool).await?;

    let v1 = Router::new()
        .route("/emails", post(routes::emails::create))
        .route_layer(middleware::from_fn_with_state(
            pool.clone(),
            auth::require_api_key,
        ));

    let app = Router::new()
        .route("/health", get(routes::health::health))
        .nest("/v1", v1)
        .layer(TraceLayer::new_for_http())
        .with_state(pool.clone());

    let addr = format!("0.0.0.0:{}", cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
