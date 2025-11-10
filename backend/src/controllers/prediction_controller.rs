use axum::{
    extract::State,
    Json,
};

use crate::{
    AppState,
    services::prediction_service::PredictionService,
    middleware::auth::AuthUser,
    errors::ApiError,
    models::prediction::Prediction,
};

pub async fn get_predictions(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let predictions = sqlx::query_as::<_, Prediction>(
        "SELECT * FROM predictions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 10"
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": predictions
    })))
}

pub async fn generate_prediction(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let service = PredictionService::new(&state.pool);
    let prediction = service.generate_prediction(user_id, None).await?;
    
    // Broadcast via WebSocket
    let _ = state.ws_tx.send(crate::websocket::WsMessage::PredictionUpdate {
        user_id,
        daily_cost: prediction.predicted_daily_cost,
        weekly_cost: prediction.predicted_weekly_cost,
        monthly_cost: prediction.predicted_monthly_cost,
    });
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": prediction
    })))
}