use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Insufficient data: {0}")]
    InsufficientData(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Internal server error: {0}")]
    InternalServerError(String),  // Add this variant
    
    #[error("JWT error: {0}")]
    JwtError(String),
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> Self {
        ApiError::ValidationError(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        ApiError::JwtError(err.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
            ApiError::ValidationError(_) => (StatusCode::BAD_REQUEST, "Validation failed"),
            ApiError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            ApiError::NotFound(_) => (StatusCode::NOT_FOUND, "Resource not found"),
            ApiError::InsufficientData(_) => (StatusCode::BAD_REQUEST, "Insufficient data"),
            ApiError::Internal(_) | ApiError::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
            }
            ApiError::JwtError(_) => (StatusCode::UNAUTHORIZED, "Invalid token"),
        };

        let body = Json(json!({
            "success": false,
            "error": message,
            "details": self.to_string()
        }));

        (status, body).into_response()
    }
}