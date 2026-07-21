use redis::AsyncCommands;

use crate::{
    common::constants::cache::{
        WECHAT_LOGIN_SESSION_PREFIX, WECHAT_LOGIN_SESSION_SECONDS, WECHAT_REGISTER_CONTEXT_PREFIX,
        WECHAT_REGISTER_CONTEXT_SECONDS,
    },
    error::AppError,
    state::AppState,
    wechat::wechat_model::{WechatLoginSession, WechatRegisterContext},
};

/// 统一生成扫码登录缓存 key，避免读写两端使用不同前缀。
fn wechat_login_session_cache_key(session_id: &str) -> String {
    format!("{WECHAT_LOGIN_SESSION_PREFIX}{session_id}")
}

/// 生成微信扫码注册上下文缓存 key。
fn wechat_register_context_cache_key(register_token: &str) -> String {
    format!("{WECHAT_REGISTER_CONTEXT_PREFIX}{register_token}")
}

/// 保存微信扫码登录会话。
///
/// 扫码状态依赖 Redis，因此 Redis 不可用时不能静默跳过。
pub async fn set_wechat_login_session(
    state: &AppState,
    session: &WechatLoginSession,
) -> Result<(), AppError> {
    let mut redis = state
        .redis
        .clone()
        .ok_or_else(|| AppError::Internal("微信扫码登录缓存不可用".to_string()))?;
    let json = serde_json::to_string(session)?;
    let _: () = redis
        .set_ex(
            wechat_login_session_cache_key(&session.session_id),
            json,
            WECHAT_LOGIN_SESSION_SECONDS,
        )
        .await?;
    tracing::debug!(
        session_id = %session.session_id,
        ttl_seconds = WECHAT_LOGIN_SESSION_SECONDS,
        "wechat qrcode login session cached"
    );
    Ok(())
}

/// 查询微信扫码登录会话。
///
/// Redis 中不存在对应 key，表示二维码已经过期。
pub async fn get_wechat_login_session(
    state: &AppState,
    session_id: &str,
) -> Result<Option<WechatLoginSession>, AppError> {
    let mut redis = state
        .redis
        .clone()
        .ok_or_else(|| AppError::Internal("微信扫码登录缓存不可用".to_string()))?;
    let cached: Option<String> = redis
        .get(wechat_login_session_cache_key(session_id))
        .await?;
    tracing::debug!(
        session_id = %session_id,
        cache_hit = cached.is_some(),
        "wechat qrcode login session cache queried"
    );
    cached
        .map(|json| serde_json::from_str::<WechatLoginSession>(&json))
        .transpose()
        .map_err(AppError::from)
}

/// 保存微信扫码注册上下文。
///
/// registerToken 是一次性凭证，日志中不能输出其完整值。
pub async fn set_wechat_register_context(
    state: &AppState,
    register_token: &str,
    context: &WechatRegisterContext,
) -> Result<(), AppError> {
    let mut redis = state
        .redis
        .clone()
        .ok_or_else(|| AppError::Internal("微信扫码注册缓存不可用".to_string()))?;
    let json = serde_json::to_string(context)?;
    let _: () = redis
        .set_ex(
            wechat_register_context_cache_key(register_token),
            json,
            WECHAT_REGISTER_CONTEXT_SECONDS,
        )
        .await?;
    tracing::debug!(
        session_id = %context.session_id,
        ttl_seconds = WECHAT_REGISTER_CONTEXT_SECONDS,
        "wechat register context cached"
    );
    Ok(())
}

/// 根据一次性注册凭证读取微信身份。
pub async fn get_wechat_register_context(
    state: &AppState,
    register_token: &str,
) -> Result<Option<WechatRegisterContext>, AppError> {
    let mut redis = state
        .redis
        .clone()
        .ok_or_else(|| AppError::Internal("微信扫码注册缓存不可用".to_string()))?;
    let cached: Option<String> = redis
        .get(wechat_register_context_cache_key(register_token))
        .await?;
    tracing::debug!(
        cache_hit = cached.is_some(),
        "wechat register context queried"
    );
    cached
        .map(|json| serde_json::from_str::<WechatRegisterContext>(&json))
        .transpose()
        .map_err(AppError::from)
}

/// 删除已使用的微信扫码注册凭证。
pub async fn delete_wechat_register_context(
    state: &AppState,
    register_token: &str,
) -> Result<(), AppError> {
    let mut redis = state
        .redis
        .clone()
        .ok_or_else(|| AppError::Internal("微信扫码注册缓存不可用".to_string()))?;
    let _: () = redis
        .del(wechat_register_context_cache_key(register_token))
        .await?;
    tracing::debug!("wechat register context deleted");
    Ok(())
}
