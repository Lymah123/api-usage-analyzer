use sqlx::PgPool;
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{
    models::prediction::Prediction,
    db::repositories::UsageRepository,
    errors::ApiError,
};

pub struct PredictionService<'a> {
    pool: &'a PgPool,
}

impl<'a> PredictionService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
    
    pub async fn generate_prediction(
        &self,
        user_id: Uuid,
        api_key_id: Option<Uuid>,
    ) -> Result<Prediction, ApiError> {
        let repo = UsageRepository::new(self.pool);
        
        let thirty_days_ago = Utc::now() - Duration::days(30);
        
        let daily_costs = repo
            .get_daily_costs(user_id, thirty_days_ago, api_key_id)
            .await?;
        
        if daily_costs.len() < 7 {
            return Err(ApiError::InsufficientData(
                "Need at least 7 days of data".to_string()
            ));
        }
        
        let costs: Vec<f64> = daily_costs.iter().map(|(_, cost)| *cost).collect();
        let avg_daily_cost = costs.iter().sum::<f64>() / costs.len() as f64;
        
        // Calculate trend
        let recent_window = 7.min(costs.len());
        let recent_costs: Vec<f64> = costs.iter().rev().take(recent_window).cloned().collect();
        let older_costs: Vec<f64> = costs.iter().take(recent_window).cloned().collect();
        
        let recent_avg = recent_costs.iter().sum::<f64>() / recent_costs.len() as f64;
        let older_avg = older_costs.iter().sum::<f64>() / older_costs.len() as f64;
        
        let trend_factor = if older_avg > 0.0 {
            recent_avg / older_avg
        } else {
            1.0
        };
        
        let daily_prediction = avg_daily_cost * trend_factor;
        let weekly_prediction = daily_prediction * 7.0;
        let monthly_prediction = daily_prediction * 30.0;
        
        let variance = self.calculate_variance(&costs);
        let coefficient_of_variation = if avg_daily_cost > 0.0 {
            variance / avg_daily_cost
        } else {
            1.0
        };
        
        let confidence = (1.0 - coefficient_of_variation.min(0.5)).max(0.5).min(0.95);
        
        let prediction = sqlx::query_as::<_, Prediction>(
            r#"
            INSERT INTO predictions (
                id, user_id, api_key_id, prediction_date,
                predicted_daily_cost, predicted_weekly_cost, predicted_monthly_cost,
                confidence_score, model_used, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(api_key_id)
        .bind(Utc::now())
        .bind(daily_prediction)
        .bind(weekly_prediction)
        .bind(monthly_prediction)
        .bind(confidence)
        .bind("linear_regression")
        .bind(Utc::now())
        .fetch_one(self.pool)
        .await?;
        
        tracing::info!(
            "Generated prediction for user {}: ${:.2}/day",
            user_id,
            daily_prediction
        );
        
        Ok(prediction)
    }
    
    fn calculate_variance(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let squared_diffs: f64 = values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum();
        
        (squared_diffs / values.len() as f64).sqrt()
    }
}