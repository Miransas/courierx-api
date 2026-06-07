mod auth;
mod config;
mod db;
mod error;
mod routes;

use axum::Router;
use axum::extract::FromRef;
use axum::middleware;
use axum::routing::{get, post};
use config::Config;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

/// Shared application state. `PgPool` and `Config` are exposed via `FromRef`
/// so handlers can extract either with `State<...>`.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
}

impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl FromRef<AppState> for Config {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

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

    let state = AppState {
        pool: pool.clone(),
        config: cfg.clone(),
    };

    let v1 = Router::new()
        .route("/emails", post(routes::emails::create))
        .route_layer(middleware::from_fn_with_state(
            pool.clone(),
            auth::require_api_key,
        ));

    let app = Router::new()
        .route("/health", get(routes::health::health))
        .nest("/auth", routes::auth::router())
        .nest("/v1", v1)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
