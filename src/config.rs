use crate::common::constants::env as env_constants;
use crate::tencent_cloud::tencent_live_license::LiveLicenseConfig;
use crate::tencent_cloud::tencent_live_model::LiveUrlConfig;
use crate::tencent_cloud::tencent_live_signer::LiveCredential;
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
    pub tencent_live_credential: Option<LiveCredential>,
    pub tencent_live_url_config: Option<LiveUrlConfig>,
    pub tencent_live_license_config: Option<LiveLicenseConfig>,

    /// 微信服务器推送消息校验 Token。
    ///
    /// 不强制启动时必须配置，避免本地开发时因为没接微信导致整个服务起不来。
    /// 真正请求 /wechat/callback 时，如果没配置，会返回 BadRequest。
    pub wechat_token: Option<String>,
    pub wechat_app_id: Option<String>,
    pub wechat_app_secret: Option<String>,
    pub wechat_encoding_aes_key: Option<String>,
    pub wechat_access_token_expire_seconds: Option<i64>,
    /// 前端 H5 基础地址，用于微信 OAuth 完成后重定向回前端。
    pub web_base_url: String,
    pub wechat_oauth_callback_base_url: Option<String>,
}

impl Settings {
    /// 从环境变量读取运行配置，并做必要的类型转换。
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            server_addr: env::var(env_constants::SERVER_ADDR)
                .unwrap_or_else(|_| env_constants::DEFAULT_SERVER_ADDR.to_string()),

            database_url: env::var(env_constants::DATABASE_URL).context("missing DATABASE_URL")?,

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

            tencent_live_credential: live_credential_from_env()?,
            tencent_live_url_config: live_url_config_from_env()?,
            tencent_live_license_config: live_license_config_from_env()?,

            wechat_token: optional_env(env_constants::WECHAT_TOKEN),
            wechat_app_id: optional_env(env_constants::WECHAT_APP_ID),
            wechat_app_secret: optional_env(env_constants::WECHAT_APP_SECRET),
            wechat_encoding_aes_key: optional_env(env_constants::WECHAT_ENCODING_AES_KEY),
            wechat_access_token_expire_seconds: Some(
                optional_env(env_constants::WECHAT_ACCESS_TOKEN_EXPIRE_SECONDS)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(env_constants::DEFAULT_WECHAT_ACCESS_TOKEN_EXPIRE_SECONDS),
            ),
            web_base_url: env::var(env_constants::WEB_BASE_URL)
                .unwrap_or_else(|_| env_constants::DEFAULT_WEB_BASE_URL.to_string()),
            wechat_oauth_callback_base_url: optional_env(
                env_constants::WECHAT_OAUTH_CALLBACK_BASE_URL,
            ),
        })
    }
}

/// 读取 Web 播放器 License 配置；URL 和 Key 必须成对配置。
fn live_license_config_from_env() -> Result<Option<LiveLicenseConfig>> {
    let url = optional_env(env_constants::TENCENT_LIVE_LICENSE_URL);
    let key = optional_env(env_constants::TENCENT_LIVE_LICENSE_KEY);

    match (url, key) {
        (Some(url), Some(key)) => Ok(Some(LiveLicenseConfig { url, key })),
        (None, None) => Ok(None),
        _ => anyhow::bail!(
            "TENCENT_LIVE_LICENSE_URL and TENCENT_LIVE_LICENSE_KEY must be configured together"
        ),
    }
}

/// 读取腾讯云直播 OpenAPI 凭证；未配置时返回 None。
fn live_credential_from_env() -> Result<Option<LiveCredential>> {
    let secret_id = env::var(env_constants::TENCENT_LIVE_SECRET_ID).ok();
    let secret_key = env::var(env_constants::TENCENT_LIVE_SECRET_KEY).ok();

    match (secret_id, secret_key) {
        (Some(secret_id), Some(secret_key))
            if !secret_id.trim().is_empty() && !secret_key.trim().is_empty() =>
        {
            Ok(Some(LiveCredential {
                secret_id,
                secret_key,
            }))
        }
        (None, None) => Ok(None),
        _ => anyhow::bail!(
            "TENCENT_LIVE_SECRET_ID and TENCENT_LIVE_SECRET_KEY must be configured together"
        ),
    }
}

/// 读取腾讯云直播推流/播放 URL 生成配置。
fn live_url_config_from_env() -> Result<Option<LiveUrlConfig>> {
    let app_name = optional_env(env_constants::TENCENT_LIVE_APP_NAME);
    let push_domain = optional_env(env_constants::TENCENT_LIVE_PUSH_DOMAIN);
    let play_domain = optional_env(env_constants::TENCENT_LIVE_PLAY_DOMAIN);
    let push_key = optional_env(env_constants::TENCENT_LIVE_PUSH_KEY);
    let play_key = optional_env(env_constants::TENCENT_LIVE_PLAY_KEY);

    let configured_count = [
        app_name.as_ref(),
        push_domain.as_ref(),
        play_domain.as_ref(),
        push_key.as_ref(),
        play_key.as_ref(),
    ]
    .into_iter()
    .filter(|value| value.is_some())
    .count();

    if configured_count == 0 {
        return Ok(None);
    }
    if configured_count != 5 {
        anyhow::bail!(
            "TENCENT_LIVE_APP_NAME, TENCENT_LIVE_PUSH_DOMAIN, TENCENT_LIVE_PLAY_DOMAIN, TENCENT_LIVE_PUSH_KEY and TENCENT_LIVE_PLAY_KEY must be configured together"
        );
    }

    let default_ttl_seconds = env::var(env_constants::TENCENT_LIVE_DEFAULT_TTL_SECONDS)
        .unwrap_or_else(|_| env_constants::DEFAULT_TENCENT_LIVE_DEFAULT_TTL_SECONDS.to_string())
        .parse()
        .context("invalid TENCENT_LIVE_DEFAULT_TTL_SECONDS")?;

    Ok(Some(LiveUrlConfig {
        app_name: app_name.expect("checked configured_count"),
        push_domain: push_domain.expect("checked configured_count"),
        play_domain: play_domain.expect("checked configured_count"),
        push_key: push_key.expect("checked configured_count"),
        play_key: play_key.expect("checked configured_count"),
        default_ttl_seconds,
    }))
}

/// 读取非空环境变量，空字符串按未配置处理。
fn optional_env(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}
