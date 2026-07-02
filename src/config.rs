use crate::common::constants::env as env_constants;
use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct Settings {
    pub server_addr: String,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret_base64: String,
    pub jwt_issuer: String,
    pub jwt_ttl_seconds: i64,
    pub mysql_max_connections: u32,
    pub http_timeout_seconds: u64,
}

impl Settings {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            server_addr: env::var(env_constants::SERVER_ADDR)
                .unwrap_or_else(|_| env_constants::DEFAULT_SERVER_ADDR.to_string()),

            database_url: env::var(env_constants::DATABASE_URL)
                .context("missing DATABASE_URL")?,

            redis_url: env::var(env_constants::REDIS_URL)
                .unwrap_or_else(|_| env_constants::DEFAULT_REDIS_URL.to_string()),

            jwt_secret_base64: env::var(env_constants::JWT_SECRET_BASE64)
                .context("missing JWT_SECRET_BASE64")?,

            jwt_issuer: env::var(env_constants::JWT_ISSUER)
                .unwrap_or_else(|_| env_constants::DEFAULT_JWT_ISSUER.to_string()),

            jwt_ttl_seconds: env::var(env_constants::JWT_TTL_SECONDS)
                .unwrap_or_else(|_| env_constants::DEFAULT_JWT_TTL_SECONDS.to_string())
                .parse()
                .context("invalid JWT_TTL_SECONDS")?,

            mysql_max_connections: env::var(env_constants::MYSQL_MAX_CONNECTIONS)
                .unwrap_or_else(|_| env_constants::DEFAULT_MYSQL_MAX_CONNECTIONS.to_string())
                .parse()
                .context("invalid MYSQL_MAX_CONNECTIONS")?,

            http_timeout_seconds: env::var(env_constants::HTTP_TIMEOUT_SECONDS)
                .unwrap_or_else(|_| env_constants::DEFAULT_HTTP_TIMEOUT_SECONDS.to_string())
                .parse()
                .context("invalid HTTP_TIMEOUT_SECONDS")?,
        })
    }
}
