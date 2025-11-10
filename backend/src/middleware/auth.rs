use axum::{
    extract::{Request, FromRequestParts},
    http::{header::AUTHORIZATION, StatusCode},        
    middleware::Next,
    response::Response,
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};  
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::ApiError;

#[derive(Debug, Serialize, Deserialize, Clone)]       
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Clone)]
pub struct AuthUser(pub Uuid);

impl AuthUser {
    pub fn user_id(&self) -> Uuid {
        self.0
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,       
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": "Missing authorization header"
                    }))
                )
            })?;

        if !auth_header.starts_with("Bearer ") {      
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Invalid authorization format"
                }))
            ));
        }

        let token = &auth_header[7..];

        let user_id = verify_token(token).map_err(|e| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": format!("Invalid token: {}", e)
                }))
            )
        })?;

        Ok(AuthUser(user_id))
    }
}

pub struct RequireAuth;

impl RequireAuth {
    pub async fn middleware(
        request: Request,
        next: Next,
    ) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
        Ok(next.run(request).await)
    }
}

pub fn verify_token(token: &str) -> Result<Uuid, ApiError> {
    let secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set");

    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::default();

    // Decode the token
    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| ApiError::Unauthorized(format!("Token decode failed: {}", e)))?;

    // Parse the user ID from the subject claim
    Uuid::parse_str(&token_data.claims.sub)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID in token".to_string()))
}