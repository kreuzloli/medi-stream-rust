use crate::config::Settings;
use crate::error::AppError;
use axum::http::HeaderMap;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::common::constants::auth::BEARER_PREFIX;

#[derive(Clone)]
pub struct JwtKeys {
    issuer: String,
    ttl_seconds: i64,
    // EncodingKey 用来签发 token，DecodingKey 用来校验 token。
    // 两者都由同一个 HS256 secret 派生，和 Java 的 NimbusJwtEncoder/Decoder 对应。
    encoding: EncodingKey,
    decoding: DecodingKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    // 这些字段是 JWT 标准或业务声明：iss 签发者，sub 用户名，iat/exp 时间戳。
    pub iss: String,
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    pub roles: Vec<String>,
    pub uid: Option<u64>,
}

impl JwtKeys {
    pub fn from_settings(settings: &Settings) -> anyhow::Result<Self> {
        // Java 配置里 secret 是 Base64；Rust 这里先解码成原始字节再创建 HMAC key。
        let secret = STANDARD.decode(&settings.jwt_secret_base64)?;
        Ok(Self {
            issuer: settings.jwt_issuer.clone(),
            ttl_seconds: settings.jwt_ttl_seconds,
            encoding: EncodingKey::from_secret(&secret),
            decoding: DecodingKey::from_secret(&secret),
        })
    }

    pub fn generate_token(
        &self,
        username: &str,
        roles: Vec<String>,
        uid: Option<u64>,
    ) -> Result<String, AppError> {
        // SystemTime 转成 Unix 秒，jsonwebtoken 的 exp/iat 使用这个整数时间戳。
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| AppError::Internal(err.to_string()))?
            .as_secs() as i64;
        let claims = Claims {
            iss: self.issuer.clone(),
            sub: username.to_string(),
            iat: now,
            exp: now + self.ttl_seconds,
            roles,
            uid,
        };
        Ok(encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &self.encoding,
        )?)
    }

    pub fn decode_token(&self, token: &str) -> Result<Claims, AppError> {
        // 只允许 HS256，并校验 issuer，避免误收其他系统签发的 token。
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[self.issuer.as_str()]);
        Ok(decode::<Claims>(token, &self.decoding, &validation)?.claims)
    }

    pub fn require_headers(&self, headers: &HeaderMap) -> Result<Claims, AppError> {
        // 这里等价于 Java JwtAuthFilter 里读取 Authorization: Bearer xxx。
        let auth = headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing token".to_string()))?;
        let token = auth
            .strip_prefix(BEARER_PREFIX)
            .ok_or_else(|| AppError::Unauthorized("Invalid token".to_string()))?;
        self.decode_token(token)
    }

    pub fn get_token_from_headers(&self, headers: &HeaderMap) -> Result<String, AppError> {
        let auth = headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing token".to_string()))?;
        let token = auth
            .strip_prefix(BEARER_PREFIX)
            .ok_or_else(|| AppError::Unauthorized("Invalid token".to_string()))?;
        Ok(token.to_string())
    }
}
