use axum::{routing::get, Router};
use std::sync::Arc;

mod config;
mod db;
mod middleware;
mod routes;
mod ws;

#[tokio::main]
async fn main() {
    let config = config::Config::load().expect("Failed to load config");

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to initialize database pool");

    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Failed to create Redis client")
        .get_async_connection()
        .await
        .expect("Failed to connect to Redis");

    let app = Router::new()
        .route("/health", get(health_check))
        .merge(routes::auth::router())
        .merge(routes::workspace::router())
        .merge(routes::channel::router())
        .merge(routes::message::router())
        .merge(routes::board::router())
        .merge(ws::handler::router())
        .with_state(Arc::new AppState { db_pool, redis_client });

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Listening on {}", addr);

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health_check() -> &'static str {
    "ok"
}

pub struct AppState {
    pub db_pool: sqlx::PgPool,
    pub redis_client: redis::aio::MultiplexedConnection,
}
