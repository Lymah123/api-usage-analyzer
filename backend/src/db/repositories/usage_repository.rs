use sqlx::PgPool;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::api_usage::ApiUsage;
use crate::models::api_key::ApiKey;
use crate::controllers::usage_controller::UsageStats;
use crate::errors::ApiError;

pub struct UsageRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> UsageRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
    
    pub async fn create_usage(
        &self,
        user_id: Uuid,
        api_key_id: Uuid,
        input_tokens: i32,
        output_tokens: i32,
        total_tokens: i32,
        cost: f64,
        model_name: String,
        endpoint: Option<String>,
        status_code: Option<i32>,
        response_time_ms: Option<i32>,
        metadata: Option<serde_json::Value>,
    ) -> Result<ApiUsage, ApiError> {
        let usage = sqlx::query_as::<_, ApiUsage>(
            r#"
            INSERT INTO api_usage (
                user_id, api_key_id, timestamp, input_tokens, output_tokens,
                total_tokens, requests, errors, cost, model_name, endpoint,
                status_code, response_time_ms, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(api_key_id)
        .bind(Utc::now())
        .bind(input_tokens)
        .bind(output_tokens)
        .bind(total_tokens)
        .bind(1) // requests
        .bind(0) // errors
        .bind(cost)
        .bind(model_name)
        .bind(endpoint)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(metadata)
        .fetch_one(self.pool)
        .await?;
        
        Ok(usage)
    }
    
    pub async fn get_usage_by_date_range(
        &self,
        user_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        api_key_id: Option<Uuid>,
    ) -> Result<Vec<ApiUsage>, ApiError> {
        let query = sqlx::query_as::<_, ApiUsage>(
            r#"
            SELECT * FROM api_usage
            WHERE user_id = $1 AND timestamp BETWEEN $2 AND $3
            "#
        )
        .bind(user_id)
        .bind(start)
        .bind(end);
        
        let usage = if let Some(key_id) = api_key_id {
            sqlx::query_as::<_, ApiUsage>(
                r#"
                SELECT * FROM api_usage
                WHERE user_id = $1 AND timestamp BETWEEN $2 AND $3 AND api_key_id = $4
                ORDER BY timestamp DESC
                "#
            )
            .bind(user_id)
            .bind(start)
            .bind(end)
            .bind(key_id)
            .fetch_all(self.pool)
            .await?
        } else {
            query
                .fetch_all(self.pool)
                .await?
        };
        
        Ok(usage)
    }
    
    pub async fn calculate_stats(
        &self,
        user_id: Uuid,
        start: DateTime<Utc>,
    ) -> Result<UsageStats, ApiError> {
        let stats = sqlx::query_as::<_, (f64, i64, i64, i64, Option<f64>)>(
            r#"
            SELECT
                COALESCE(SUM(cost), 0) as total_cost,
                COALESCE(SUM(total_tokens), 0) as total_tokens,
                COALESCE(SUM(requests), 0) as total_requests,
                COALESCE(SUM(errors), 0) as total_errors,
                AVG(response_time_ms) as avg_response_time
            FROM api_usage
            WHERE user_id = $1 AND timestamp >= $2
            "#
        )
        .bind(user_id)
        .bind(start)
        .fetch_one(self.pool)
        .await?;
        
        let error_rate = if stats.2 > 0 {
            (stats.3 as f64 / stats.2 as f64) * 100.0
        } else {
            0.0
        };
        
        Ok(UsageStats {
            total_cost: stats.0,
            total_tokens: stats.1,
            total_requests: stats.2,
            total_errors: stats.3,
            error_rate,
            avg_response_time: stats.4,
        })
    }
    
    pub async fn get_api_key(&self, api_key_id: Uuid) -> Result<ApiKey, ApiError> {
        let key = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE id = $1"
        )
        .bind(api_key_id)
        .fetch_one(self.pool)
        .await?;
        
        Ok(key)
    }
    
    pub async fn get_daily_costs(
        &self,
        user_id: Uuid,
        start: DateTime<Utc>,
        api_key_id: Option<Uuid>,
    ) -> Result<Vec<(String, f64)>, ApiError> {
        let costs = if let Some(key_id) = api_key_id {
            sqlx::query_as::<_, (String, f64)>(
                r#"
                SELECT 
                    DATE(timestamp) as date,
                    SUM(cost) as total_cost
                FROM api_usage
                WHERE user_id = $1 AND timestamp >= $2 AND api_key_id = $3
                GROUP BY DATE(timestamp)
                ORDER BY DATE(timestamp) ASC
                "#
            )
            .bind(user_id)
            .bind(start)
            .bind(key_id)
            .fetch_all(self.pool)
            .await?
        } else {
            sqlx::query_as::<_, (String, f64)>(
                r#"
                SELECT 
                    DATE(timestamp) as date,
                    SUM(cost) as total_cost
                FROM api_usage
                WHERE user_id = $1 AND timestamp >= $2
                GROUP BY DATE(timestamp)
                ORDER BY DATE(timestamp) ASC
                "#
            )
            .bind(user_id)
            .bind(start)
            .fetch_all(self.pool)
            .await?
        };
        
        Ok(costs)
    }
}