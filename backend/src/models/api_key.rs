use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub provider: String,
    pub encrypted_key: String,
    pub cost_per_1k_input: f64,
    pub cost_per_1k_output: f64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
