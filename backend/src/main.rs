use api_usage_analyzer::{
    config::Config, db::create_pool, jobs::start_background_jobs, routes::create_router, AppState,
};
use axum::http::{HeaderValue, Method};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    tracing::info!("Starting API Usage Analyzer v1.0.0");

    // Create database pool
    let pool = create_pool(&config.database_url).await?;

    // Run migrations
    sqlx::migrate!().run(&pool).await?;
    tracing::info!("Database migrations completed");

    // Create Redis pool
    let redis_cfg = deadpool_redis::Config::from_url(&config.redis_url);
    let redis_pool = redis_cfg
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("Failed to create Redis pool");

    // Create WebSocket broadcast channel
    let (ws_tx, _) = tokio::sync::broadcast::channel(100);

    // Create app state
    let state = AppState {
        pool: pool.clone(),
        redis_pool,
        config: config.clone(),
        ws_tx: ws_tx.clone(),
    };

    // Start background jobs
    tokio::spawn(start_background_jobs(pool.clone(), ws_tx.clone()));

    // Build router with fixed CORS configuration
    let app = create_router(state)
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:8080".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                ])
                .allow_credentials(true),
        )
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("🚀 Server running on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
