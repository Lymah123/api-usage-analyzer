use sqlx::PgPool;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use tokio::sync::broadcast;

use crate::{
    models::api_usage::{ApiUsage, CreateUsageRequest},
    db::repositories::UsageRepository,
    controllers::usage_controller::UsageStats,
    websocket::WsMessage,
    errors::ApiError,
};

pub struct UsageService<'a> {
    pool: &'a PgPool,
    ws_tx: &'a broadcast::Sender<WsMessage>,
}

impl<'a> UsageService<'a> {
    pub fn new(pool: &'a PgPool, ws_tx: &'a broadcast::Sender<WsMessage>) -> Self {
        Self { pool, ws_tx }
    }
    
    pub async fn record_usage(
        &self,
        user_id: Uuid,
        req: CreateUsageRequest,
    ) -> Result<ApiUsage, ApiError> {
        let repo = UsageRepository::new(self.pool);
        
        // Get API key pricing
        let api_key = repo.get_api_key(req.api_key_id).await?;
        
        // Calculate cost
        let total_tokens = req.input_tokens + req.output_tokens;
        let cost = ApiUsage::calculate_cost(
            req.input_tokens,
            req.output_tokens,
            api_key.cost_per_1k_input,
            api_key.cost_per_1k_output,
        );
        
        // Create usage record
        let usage = repo.create_usage(
            user_id,
            req.api_key_id,
            req.input_tokens,
            req.output_tokens,
            total_tokens,
            cost,
            req.model_name,
            req.endpoint,
            req.status_code,
            req.response_time_ms,
            req.metadata,
        ).await?;
        
        // Broadcast WebSocket update
        let _ = self.ws_tx.send(WsMessage::UsageUpdate {
            user_id,
            cost,
            tokens: total_tokens,
            timestamp: usage.timestamp.to_rfc3339(),
        });
        
        Ok(usage)
    }
    
    pub async fn get_usage(
        &self,
        user_id: Uuid,
        start_date: Option<&str>,
        end_date: Option<&str>,
        api_key_id: Option<Uuid>,
    ) -> Result<Vec<ApiUsage>, ApiError> {
        let repo = UsageRepository::new(self.pool);
        
        let start = start_date
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|| Utc::now() - Duration::days(7));
        
        let end = end_date
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        
        repo.get_usage_by_date_range(user_id, start, end, api_key_id).await
    }
    
    pub async fn calculate_stats(
        &self,
        user_id: Uuid,
        period: &str,
    ) -> Result<UsageStats, ApiError> {
        let repo = UsageRepository::new(self.pool);
        
        let days = match period {
            "24h" | "1d" => 1,
            "7d" => 7,
            "30d" => 30,
            "90d" => 90,
            _ => 7,
        };
        
        let start = Utc::now() - Duration::days(days);
        
        repo.calculate_stats(user_id, start).await
    }
    
    pub async fn export_usage(
        &self,
        user_id: Uuid,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<String, ApiError> {
        let usage = self.get_usage(user_id, start_date, end_date, None).await?;
        
        let json = serde_json::to_string_pretty(&usage)
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        
        Ok(json)
    }
}