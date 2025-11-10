use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub server: ServerConfig,
    pub email: Option<EmailConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_password: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            database_url: env::var("DATABASE_URL")?,
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            jwt_secret: env::var("JWT_SECRET")?,
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "3000".to_string())
                    .parse()?,
            },
            email: Self::email_config(),
        })
    }
    
    fn email_config() -> Option<EmailConfig> {
        Some(EmailConfig {
            smtp_host: env::var("SMTP_HOST").ok()?,
            smtp_port: env::var("SMTP_PORT").ok()?.parse().ok()?,
            smtp_user: env::var("SMTP_USER").ok()?,
            smtp_password: env::var("SMTP_PASSWORD").ok()?,
        })
    }
}
