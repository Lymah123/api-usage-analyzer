use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Budget {
    pub id: Uuid,
    pub user_id: Uuid,
    pub api_key_id: Option<Uuid>,
    pub limit_type: String,
    pub limit_amount: f64,
    pub alert_threshold: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}