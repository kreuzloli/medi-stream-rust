use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct Settings {
    // 服务监听地址，例如 0.0.0.0:8080。
    pub server_addr: String,
    // SQLx 使用的 MySQL 连接串，例如 mysql://user:password@host:3306/db。
    pub database_url: String,
    // Redis 连接串。这里兼容带密码和不带密码两种形式。
    pub redis_url: String,
    // Base64 编码后的 HS256 密钥，和 Java 项目的 app.security.jwt.secret 对应。
    pub jwt_secret_base64: String,
    pub jwt_issuer: String,
    pub jwt_ttl_seconds: i64,
    pub mysql_max_connections: u32,
    // 外部 HTTP API 请求超时时间，单位：秒。
    pub http_timeout_seconds: u64,
}

impl Settings {
    pub fn from_env() -> Result<Self> {
        // Rust 里通常让配置来自环境变量；本项目 main.rs 会先加载 .env。
        // 必填项用 context 给出更清楚的启动失败原因。
        Ok(Self {
            server_addr: env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            database_url: env::var("DATABASE_URL").context("missing DATABASE_URL")?,
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379/0".to_string()),
            jwt_secret_base64: env::var("JWT_SECRET_BASE64")
                .context("missing JWT_SECRET_BASE64")?,
            jwt_issuer: env::var("JWT_ISSUER").unwrap_or_else(|_| "medistream".to_string()),
            jwt_ttl_seconds: env::var("JWT_TTL_SECONDS")
                .unwrap_or_else(|_| "7200".to_string())
                .parse()
                .context("invalid JWT_TTL_SECONDS")?,
            mysql_max_connections: env::var("MYSQL_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .context("invalid MYSQL_MAX_CONNECTIONS")?,
            http_timeout_seconds: env::var("HTTP_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .context("invalid HTTP_TIMEOUT_SECONDS")?,
        })
    }
}
