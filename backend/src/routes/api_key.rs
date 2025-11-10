use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;

use crate::controllers::api_key_controller;
use crate::middleware::auth::auth_middleware;

pub fn routes(pool: PgPool) -> Router {
    Router::new()
        .route("/", post(api_key_controller::create_api_key))
        .route("/", get(api_key_controller::list_api_keys))
        .route("/:key_id", get(api_key_controller::get_api_key))
        .route("/:key_id", put(api_key_controller::update_api_key))
        .route("/:key_id", delete(api_key_controller::delete_api_key))
        .route("/:key_id/rotate", post(api_key_controller::rotate_api_key))
        .route("/:key_id/stats", get(api_key_controller::get_api_key_stats))
        .layer(axum::middleware::from_fn_with_state(
            pool.clone(),
            auth_middleware,
        ))
        .with_state(pool)
}