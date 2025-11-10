use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::User;
use crate::errors::ApiError;

pub struct UserRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> UserRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
    
    pub async fn create_user(
        &self,
        email: String,
        password_hash: String,
        name: String,
    ) -> Result<User, ApiError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, password, name, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(email)
        .bind(password_hash)
        .bind(name)
        .bind(true)
        .fetch_one(self.pool)
        .await?;
        
        Ok(user)
    }
    
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, ApiError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(self.pool)
        .await?;
        
        Ok(user)
    }
    
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, ApiError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;
        
        Ok(user)
    }
}
