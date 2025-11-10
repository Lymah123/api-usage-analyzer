use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use base64::Engine;

use crate::{
    AppState,
    errors::ApiError,
    middleware::auth::AuthUser,
    models::api_key::ApiKey,
};

// ==================== REQUEST/RESPONSE TYPES ====================

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub provider: String,
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateApiKeyRequest {
    pub name: Option<String>,
    pub api_key: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListApiKeysQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_preview: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyCreateResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub provider: String,
    pub api_key: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyRotateResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub provider: String,
    pub new_key: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyListResponse {
    pub data: Vec<ApiKeyResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyUsageStats {
    pub total_requests: i64,
    pub total_cost: f64,
    pub requests_today: i64,
    pub cost_today: f64,
}

// ==================== CONVERSIONS ====================

impl From<ApiKey> for ApiKeyResponse {
    fn from(api_key: ApiKey) -> Self {
        let key_preview = if api_key.encrypted_key.len() >= 4 {
            Some(format!("****{}", &api_key.encrypted_key[api_key.encrypted_key.len() - 4..]))
        } else {
            None
        };

        Self {
            id: api_key.id,
            user_id: api_key.user_id,
            name: api_key.name,
            provider: api_key.provider,
            key_preview,
            is_active: api_key.is_active,
            created_at: api_key.created_at,
        }
    }
}

// ==================== HANDLERS ====================

/// Create a new API key
pub async fn create_api_key(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<ApiKeyCreateResponse>), ApiError> {
    let encrypted_key = encrypt_api_key(&payload.api_key)?;

    let api_key = sqlx::query_as::<_, ApiKey>(
        r#"
        INSERT INTO api_keys (user_id, name, provider, encrypted_key, is_active)
        VALUES ($1, $2, $3, $4, true)
        RETURNING *
        "#,
    )
    .bind(user)
    .bind(&payload.name)
    .bind(&payload.provider)
    .bind(&encrypted_key)
    .fetch_one(&state.pool)
    .await?;

    tracing::info!(
        "API key created - User: {}, Provider: {}, Name: {}",
        user,
        payload.provider,
        payload.name
    );

    Ok((StatusCode::CREATED, Json(ApiKeyCreateResponse {
        id: api_key.id,
        user_id: api_key.user_id,
        name: api_key.name,
        provider: api_key.provider,
        api_key: payload.api_key,
        is_active: api_key.is_active,
        created_at: api_key.created_at,
    })))
}

/// List all API keys for the authenticated user
pub async fn list_api_keys(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(params): Query<ListApiKeysQuery>,
) -> Result<Json<ApiKeyListResponse>, ApiError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(10).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let api_keys = sqlx::query_as::<_, ApiKey>(
        r#"
        SELECT * FROM api_keys
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.pool)
    .await?;

    let total: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM api_keys
        WHERE user_id = $1
        "#,
    )
    .bind(user)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(ApiKeyListResponse {
        data: api_keys.into_iter().map(|k| k.into()).collect(),
        total: total.0,
        page,
        per_page,
    }))
}

/// Get a specific API key by ID
pub async fn get_api_key(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(key_id): Path<Uuid>,
) -> Result<Json<ApiKeyResponse>, ApiError> {
    let api_key = sqlx::query_as::<_, ApiKey>(
        r#"
        SELECT * FROM api_keys
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(key_id)
    .bind(user)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound("API key not found".to_string()))?;

    Ok(Json(api_key.into()))
}

/// Update an API key
pub async fn update_api_key(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(key_id): Path<Uuid>,
    Json(payload): Json<UpdateApiKeyRequest>,
) -> Result<Json<ApiKeyResponse>, ApiError> {
    let encrypted_key = if let Some(ref key) = payload.api_key {
        Some(encrypt_api_key(key)?)
    } else {
        None
    };

    let updated_key = sqlx::query_as::<_, ApiKey>(
        r#"
        UPDATE api_keys
        SET 
            name = COALESCE($1, name),
            encrypted_key = COALESCE($2, encrypted_key),
            is_active = COALESCE($3, is_active),
            updated_at = NOW()
        WHERE id = $4 AND user_id = $5
        RETURNING *
        "#,
    )
    .bind(payload.name)
    .bind(encrypted_key)
    .bind(payload.is_active)
    .bind(key_id)
    .bind(user)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound("API key not found".to_string()))?;

    tracing::info!("API key updated - User: {}, Key ID: {}", user, key_id);

    Ok(Json(updated_key.into()))
}

/// Delete (deactivate) an API key
pub async fn delete_api_key(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(key_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query(
        r#"
        UPDATE api_keys
        SET is_active = false, updated_at = NOW()
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(key_id)
    .bind(user)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("API key not found".to_string()));
    }

    tracing::info!("API key deactivated - User: {}, Key ID: {}", user, key_id);

    Ok(StatusCode::NO_CONTENT)
}

/// Rotate an API key (generate a new key value)
pub async fn rotate_api_key(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(key_id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiKeyRotateResponse>), ApiError> {
    use rand::RngCore;
    use rand::rngs::OsRng;
    use sha2::{Sha256, Digest};

    // Verify the API key exists and belongs to the user
    let existing_key = sqlx::query_as::<_, ApiKey>(
        r#"
        SELECT * FROM api_keys
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(key_id)
    .bind(user)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound("API key not found".to_string()))?;

    // Generate a new random API key using OsRng (which is Send)
    let mut random_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut random_bytes);

    let mut hasher = Sha256::new();
    hasher.update(&random_bytes);
    hasher.update(user.as_bytes());
    hasher.update(chrono::Utc::now().timestamp().to_le_bytes());
    let hash_result = hasher.finalize();
    
    let new_key = format!(
        "sk_live_{}",
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(&hash_result[..24])
            .replace('-', "")
            .replace('_', "")
    );

    // Encrypt the new key
    let encrypted_key = encrypt_api_key(&new_key)?;

    // Update the key in the database
    let updated = sqlx::query_as::<_, ApiKey>(
        r#"
        UPDATE api_keys
        SET 
            encrypted_key = $1, 
            updated_at = NOW()
        WHERE id = $2 AND user_id = $3
        RETURNING *
        "#,
    )
    .bind(&encrypted_key)
    .bind(key_id)
    .bind(user)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound("API key not found".to_string()))?;

    tracing::info!(
        "API key rotated - User: {}, Key ID: {}, Provider: {}",
        user,
        key_id,
        existing_key.provider
    );

    Ok((StatusCode::OK, Json(ApiKeyRotateResponse {
        id: updated.id,
        user_id: updated.user_id,
        name: updated.name,
        provider: updated.provider,
        new_key, 
        is_active: updated.is_active,
        created_at: updated.created_at,
    })))
}

/// Get usage statistics for an API key
pub async fn get_api_key_stats(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(key_id): Path<Uuid>,
) -> Result<Json<ApiKeyUsageStats>, ApiError> {
    let _api_key = sqlx::query_as::<_, ApiKey>(
        r#"
        SELECT * FROM api_keys
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(key_id)
    .bind(user)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound("API key not found".to_string()))?;

    let stats = sqlx::query_as::<_, (i64, f64, i64, f64)>(
        r#"
        SELECT 
            COUNT(*)::bigint as total_requests,
            COALESCE(SUM(cost), 0)::float8 as total_cost,
            COUNT(CASE WHEN timestamp >= CURRENT_DATE THEN 1 END)::bigint as requests_today,
            COALESCE(SUM(CASE WHEN timestamp >= CURRENT_DATE THEN cost ELSE 0 END), 0)::float8 as cost_today
        FROM api_usage
        WHERE api_key_id = $1
        "#,
    )
    .bind(key_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(ApiKeyUsageStats {
        total_requests: stats.0,
        total_cost: stats.1,
        requests_today: stats.2,
        cost_today: stats.3,
    }))
}

// ==================== ENCRYPTION HELPERS ====================

fn encrypt_api_key(key: &str) -> Result<String, ApiError> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    
    let encryption_key = std::env::var("API_KEY_ENCRYPTION_KEY")
        .unwrap_or_else(|_| {
            tracing::warn!("Using default encryption key - NOT SECURE FOR PRODUCTION");
            "your-32-byte-encryption-key-here!".to_string()
        });
    
    let mut key_bytes = [0u8; 32];
    let enc_key_bytes = encryption_key.as_bytes();
    let len = enc_key_bytes.len().min(32);
    key_bytes[..len].copy_from_slice(&enc_key_bytes[..len]);
    
    let cipher = Aes256Gcm::new(&key_bytes.into());
    
    let mut nonce_bytes = [0u8; 12];
    getrandom::getrandom(&mut nonce_bytes)
        .map_err(|e| ApiError::InternalServerError(format!("Failed to generate nonce: {}", e)))?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, key.as_bytes())
        .map_err(|e| ApiError::InternalServerError(format!("Encryption failed: {}", e)))?;
    
    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);
    
    Ok(base64::engine::general_purpose::STANDARD.encode(&combined))
}

#[allow(dead_code)]
pub fn decrypt_api_key(encrypted: &str) -> Result<String, ApiError> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    
    let encryption_key = std::env::var("API_KEY_ENCRYPTION_KEY")
        .unwrap_or_else(|_| "your-32-byte-encryption-key-here!".to_string());
    
    let mut key_bytes = [0u8; 32];
    let enc_key_bytes = encryption_key.as_bytes();
    let len = enc_key_bytes.len().min(32);
    key_bytes[..len].copy_from_slice(&enc_key_bytes[..len]);
    
    let cipher = Aes256Gcm::new(&key_bytes.into());
    
    let combined = base64::engine::general_purpose::STANDARD
        .decode(encrypted)
        .map_err(|e| ApiError::InternalServerError(format!("Base64 decode failed: {}", e)))?;
    
    if combined.len() < 12 {
        return Err(ApiError::InternalServerError("Invalid encrypted data".to_string()));
    }
    
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| ApiError::InternalServerError(format!("Decryption failed: {}", e)))?;
    
    String::from_utf8(plaintext)
        .map_err(|e| ApiError::InternalServerError(format!("Invalid UTF-8: {}", e)))
}