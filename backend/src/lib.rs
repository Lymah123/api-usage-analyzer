pub mod config;
pub mod db;
pub mod models;
pub mod routes;
pub mod controllers;
pub mod services;
pub mod middleware;
pub mod errors;
pub mod utils;
pub mod websocket;
pub mod jobs;

use sqlx::PgPool;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub redis_pool: deadpool_redis::Pool,
    pub config: config::Config,
    pub ws_tx: broadcast::Sender<websocket::WsMessage>,
}