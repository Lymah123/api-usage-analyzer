use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Serialize;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use jsonwebtoken::{encode, Header, EncodingKey};

use crate::{
    AppState,
    models::user::{CreateUserRequest, LoginRequest, User},
    db::repositories::UserRepository,
    middleware::auth::{Claims, AuthUser},
    errors::ApiError,
};

#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub data: AuthData,
}

#[derive(Serialize)]
pub struct AuthData {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: uuid::Uuid,
    pub email: String,
    pub name: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), ApiError> {
    let repo = UserRepository::new(&state.pool);
    
    // Check if user exists
    if repo.find_by_email(&req.email).await?.is_some() {
        return Err(ApiError::ValidationError("Email already registered".to_string()));
    }
    
    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .to_string();
    
    // Create user
    let user = repo.create_user(req.email, password_hash, req.name).await?;
    
    // Generate JWT
    let token = generate_token(&user, &state.config.jwt_secret)?;
    
    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            success: true,
            data: AuthData {
                token,
                user: UserResponse {
                    id: user.id,
                    email: user.email,
                    name: user.name,
                },
            },
        })
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let repo = UserRepository::new(&state.pool);
    
    // Find user
    let user = repo
        .find_by_email(&req.email)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Invalid credentials".to_string()))?;
    
    // Verify password
    let parsed_hash = PasswordHash::new(&user.password)
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| ApiError::Unauthorized("Invalid credentials".to_string()))?;
    
    // Generate JWT
    let token = generate_token(&user, &state.config.jwt_secret)?;
    
    Ok(Json(AuthResponse {
        success: true,
        data: AuthData {
            token,
            user: UserResponse {
                id: user.id,
                email: user.email,
                name: user.name,
            },
        },
    }))
}

pub async fn get_current_user(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let repo = UserRepository::new(&state.pool);
    
    let user = repo
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
        }
    })))
}

fn generate_token(user: &User, secret: &str) -> Result<String, ApiError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .expect("valid timestamp")
        .timestamp() as usize;
    
    let claims = Claims {
        sub: user.id.to_string(),
        exp: expiration,
        iat: chrono::Utc::now().timestamp() as usize,
    };
    
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes())
    )?;
    
    Ok(token)
}
