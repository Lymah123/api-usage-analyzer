use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Prediction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub api_key_id: Option<Uuid>,
    pub prediction_date: DateTime<Utc>,
    pub predicted_daily_cost: f64,
    pub predicted_weekly_cost: f64,
    pub predicted_monthly_cost: f64,
    pub confidence_score: f64,
    pub model_used: String,
    pub created_at: DateTime<Utc>,
}
