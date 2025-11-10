use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Alert {
    pub id: Uuid,
    pub user_id: Uuid,
    pub api_key_id: Option<Uuid>,
    pub alert_type: String,
    pub severity: String,
    pub threshold_value: Option<f64>,
    pub current_value: Option<f64>,
    pub message: String,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}