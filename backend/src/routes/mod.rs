use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::controllers::{
    analytics_controller, api_key_controller, auth_controller, prediction_controller,
    usage_controller,
};
use crate::middleware::auth::RequireAuth;
use crate::websocket::websocket_handler;
use crate::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/ws", get(websocket_handler))
        .nest("/api/v1", api_routes())
        .with_state(state)
}

fn api_routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth_routes())
        .nest("/usage", usage_routes())
        .nest("/predictions", prediction_routes())
        .nest("/analytics", analytics_routes())
        .nest("/api-keys", api_key_routes())
}

fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(auth_controller::register))
        .route("/login", post(auth_controller::login))
        .route("/me", get(auth_controller::get_current_user))
}

fn usage_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(usage_controller::record_usage))
        .route("/", get(usage_controller::get_usage))
        .route("/stats", get(usage_controller::get_stats))
        .route("/export", get(usage_controller::export_usage))
        .layer(axum::middleware::from_fn_with_state(
            (),
            RequireAuth::middleware,
        ))
}

fn prediction_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(prediction_controller::get_predictions))
        .route(
            "/generate",
            post(prediction_controller::generate_prediction),
        )
        .layer(axum::middleware::from_fn_with_state(
            (),
            RequireAuth::middleware,
        ))
}

fn analytics_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/overview",
            get(analytics_controller::get_analytics_overview),
        )
        .route("/anomalies", get(analytics_controller::detect_anomalies))
        .layer(axum::middleware::from_fn_with_state(
            (),
            RequireAuth::middleware,
        ))
}

fn api_key_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(api_key_controller::create_api_key))
        .route("/", get(api_key_controller::list_api_keys))
        .route("/:key_id", get(api_key_controller::get_api_key))
        .route("/:key_id", put(api_key_controller::update_api_key))
        .route("/:key_id", delete(api_key_controller::delete_api_key))
        .route("/:key_id/stats", get(api_key_controller::get_api_key_stats))
        .route("/:key_id/rotate", post(|state, user, path| async move {
            api_key_controller::rotate_api_key(state, user, path).await
        }))
        .layer(axum::middleware::from_fn_with_state(
            (),
            RequireAuth::middleware,
        ))
}

async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    }))
}