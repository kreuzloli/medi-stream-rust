use crate::common::constants::auth::BEARER_PREFIX;
use crate::config::Settings;
use crate::error::AppError;
use axum::async_trait;
use axum::extract::{FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct JwtKeys {
    issuer: String,
    ttl_seconds: u64,
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
    /// 根据应用配置创建 JWT 编码和解码密钥。
    pub fn from_settings(settings: &Settings) -> anyhow::Result<Self> {
        anyhow::ensure!(
            settings.jwt_ttl_seconds > 0,
            "JWT_TTL_SECONDS must be greater than zero"
        );
        // Java 配置里 secret 是 Base64；Rust 这里先解码成原始字节再创建 HMAC key。
        let secret = STANDARD.decode(&settings.jwt_secret_base64)?;
        Ok(Self {
            issuer: settings.jwt_issuer.clone(),
            ttl_seconds: settings.jwt_ttl_seconds as u64,
            encoding: EncodingKey::from_secret(&secret),
            decoding: DecodingKey::from_secret(&secret),
        })
    }

    /// 根据用户名、角色和用户 ID 签发 JWT。
    pub fn generate_token(
        &self,
        subject: &str,
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
            sub: subject.to_string(),
            iat: now,
            exp: now + self.ttl_seconds as i64,
            roles,
            uid,
        };
        Ok(encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &self.encoding,
        )?)
    }

    /// 解析并校验输入数据。
    pub fn decode_token(&self, token: &str) -> Result<Claims, AppError> {
        // 只允许 HS256，并校验 issuer，避免误收其他系统签发的 token。
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[self.issuer.as_str()]);
        Ok(decode::<Claims>(token, &self.decoding, &validation)?.claims)
    }

    /// 返回 Token 缓存应使用的有效期，确保 Redis 会话与 JWT 同时过期。
    pub fn token_ttl_seconds(&self) -> u64 {
        self.ttl_seconds
    }
}

#[derive(Debug, Clone)]
pub struct CurrentUser(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    /// 读取认证中间件写入的当前用户上下文。
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CurrentUser>()
            .cloned()
            .ok_or_else(|| AppError::Unauthorized("Missing token".to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct CurrentToken(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for CurrentToken
where
    S: Send + Sync,
{
    type Rejection = AppError;

    /// 读取认证中间件保存的 Token 原文，仅供当前会话注销使用。
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CurrentToken>()
            .cloned()
            .ok_or_else(|| AppError::Unauthorized("Missing token".to_string()))
    }
}

/// 统一校验 JWT 和 Redis Token 状态，并把当前用户上下文传给后续 Handler。
pub async fn authenticate_user(
    State(state): State<crate::state::AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing token".to_string()))?
        .strip_prefix(BEARER_PREFIX)
        .map(str::to_owned)
        .ok_or_else(|| AppError::Unauthorized("Invalid token".to_string()))?;
    let claims = state.jwt.decode_token(&token)?;
    crate::common::cache::require_cached_token(&state, &token).await?;
    request.extensions_mut().insert(CurrentUser(claims));
    request.extensions_mut().insert(CurrentToken(token));
    Ok(next.run(request).await)
}
