use axum::{
    extract::{State, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    models::api_usage::{ApiUsage, CreateUsageRequest},
    services::usage_service::UsageService,
    middleware::auth::AuthUser,
    errors::ApiError,
};

#[derive(Debug, Deserialize)]
pub struct UsageQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub api_key_id: Option<Uuid>,
    pub period: Option<String>,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
}

#[derive(Serialize)]
pub struct UsageStats {
    pub total_cost: f64,
    pub total_tokens: i64,
    pub total_requests: i64,
    pub total_errors: i64,
    pub error_rate: f64,
    pub avg_response_time: Option<f64>,
}

pub async fn record_usage(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateUsageRequest>,
) -> Result<(StatusCode, Json<ApiResponse<ApiUsage>>), ApiError> {
    req.validate()?;
    
    let service = UsageService::new(&state.pool, &state.ws_tx);
    let usage = service.record_usage(user_id, req).await?;
    
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            data: usage,
        })
    ))
}

pub async fn get_usage(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(query): Query<UsageQuery>,
) -> Result<Json<ApiResponse<Vec<ApiUsage>>>, ApiError> {
    let service = UsageService::new(&state.pool, &state.ws_tx);
    
    let usage = service
        .get_usage(
            user_id,
            query.start_date.as_deref(),
            query.end_date.as_deref(),
            query.api_key_id,
        )
        .await?;
    
    Ok(Json(ApiResponse {
        success: true,
        data: usage,
    }))
}

pub async fn get_stats(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(query): Query<UsageQuery>,
) -> Result<Json<ApiResponse<UsageStats>>, ApiError> {
    let service = UsageService::new(&state.pool, &state.ws_tx);
    let period = query.period.as_deref().unwrap_or("7d");
    
    let stats = service.calculate_stats(user_id, period).await?;
    
    Ok(Json(ApiResponse {
        success: true,
        data: stats,
    }))
}

pub async fn export_usage(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(query): Query<UsageQuery>,
) -> Result<(StatusCode, String), ApiError> {
    let service = UsageService::new(&state.pool, &state.ws_tx);
    
    let data = service
        .export_usage(
            user_id,
            query.start_date.as_deref(),
            query.end_date.as_deref(),
        )
        .await?;
    
    Ok((StatusCode::OK, data))
}