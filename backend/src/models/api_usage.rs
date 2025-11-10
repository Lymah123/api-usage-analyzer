use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiUsage {
    pub id: i64,
    pub user_id: Uuid,
    pub api_key_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub total_tokens: i32,
    pub requests: i32,
    pub errors: i32,
    pub cost: f64,
    pub model_name: Option<String>,
    pub endpoint: Option<String>,
    pub status_code: Option<i32>,
    pub response_time_ms: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUsageRequest {
    pub api_key_id: Uuid,
    
    #[validate(range(min = 0))]
    pub input_tokens: i32,
    
    #[validate(range(min = 0))]
    pub output_tokens: i32,
    
    #[validate(length(min = 1, max = 100))]
    pub model_name: String,
    
    pub endpoint: Option<String>,
    
    #[validate(range(min = 100, max = 599))]
    pub status_code: Option<i32>,
    
    #[validate(range(min = 0))]
    pub response_time_ms: Option<i32>,
    
    pub metadata: Option<serde_json::Value>,
}

impl ApiUsage {
    pub fn calculate_cost(
        input_tokens: i32,
        output_tokens: i32,
        cost_per_1k_input: f64,
        cost_per_1k_output: f64,
    ) -> f64 {
        let input_cost = (input_tokens as f64 / 1000.0) * cost_per_1k_input;
        let output_cost = (output_tokens as f64 / 1000.0) * cost_per_1k_output;
        
        ((input_cost + output_cost) * 100.0).round() / 100.0
    }
}
