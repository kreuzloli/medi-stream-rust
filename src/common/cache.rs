use crate::account::account_model::{AccountDetail, LoginType};
use crate::common::constants::cache::{
    ACCOUNT_CACHE_SECONDS, ACCOUNT_DETAIL_CACHE_PREFIX, LOGIN_VERIFICATION_CODE_PREFIX,
    TOKEN_CACHE_PREFIX,
};
use crate::error::AppError;
use crate::state::AppState;
use redis::AsyncCommands;

/// 从 Redis 读取账号详情缓存；缓存不存在或不可解析时返回 None。
pub async fn get_account(state: &mut AppState, id: u64) -> Result<Option<AccountDetail>, AppError> {
    if let Some(redis) = state.redis.as_mut() {
        let cached: Option<String> = redis.get(account_cache_key(id)).await?;
        if let Some(cached) = cached {
            if let Ok(account) = serde_json::from_str::<AccountDetail>(&cached) {
                return Ok(Some(account));
            }
        }
    }
    Ok(None)
}

/// 写入缓存，减少后续数据库访问。
pub async fn cache_account(state: &mut AppState, account: &AccountDetail) -> Result<(), AppError> {
    let Some(id) = account.profile.id else {
        return Ok(());
    };
    if let Some(redis) = state.redis.as_mut() {
        let json = serde_json::to_string(account)?;
        let _: () = redis
            .set_ex(account_cache_key(id), json, ACCOUNT_CACHE_SECONDS)
            .await?;
    }
    Ok(())
}

/// 写入缓存，减少后续数据库访问。
pub async fn cache_token(
    state: &mut AppState,
    account: &AccountDetail,
    token: &str,
) -> Result<(), AppError> {
    let key = token_cache_key(token);
    if let Some(redis) = state.redis.as_mut() {
        let json = serde_json::to_string(account)?;
        let _: () = redis.set_ex(key, json, ACCOUNT_CACHE_SECONDS).await?;
    }
    Ok(())
}

/// 删除账号详情缓存；Redis 不可用时直接跳过。
pub async fn delete_account_cache(state: &mut AppState, id: u64) -> Result<(), AppError> {
    if let Some(redis) = state.redis.as_mut() {
        let _: () = redis.del(account_cache_key(id)).await?;
    }
    Ok(())
}

/// 删除 token 缓存；Redis 不可用时直接跳过。
pub async fn delete_token_cache(state: &mut AppState, token: &str) -> Result<(), AppError> {
    let key = token_cache_key(token);
    if let Some(redis) = state.redis.as_mut() {
        let _: () = redis.del(key).await?;
    }
    Ok(())
}

/// 验证凭证或验证码是否有效。
pub async fn verify_login_verification_code(
    state: &mut AppState,
    login_type: LoginType,
    login_identifier: &str,
    verification_code: &str,
) -> Result<(), AppError> {
    let Some(redis) = state.redis.as_mut() else {
        return Err(AppError::Internal("验证码服务不可用".to_string()));
    };

    let key = login_verification_code_key(login_type, login_identifier);
    let cached: Option<String> = redis.get(&key).await?;
    if cached.as_deref() != Some(verification_code) {
        return Err(AppError::Unauthorized("验证码错误".to_string()));
    }

    let _: () = redis.del(key).await?;
    Ok(())
}

/// 处理账号相关的业务转换。
fn account_cache_key(id: u64) -> String {
    format!("{ACCOUNT_DETAIL_CACHE_PREFIX}{id}")
}

/// 生成 token 缓存 key。
fn token_cache_key(token: &str) -> String {
    format!("{TOKEN_CACHE_PREFIX}{token}")
}

/// 处理登录相关的业务转换。
fn login_verification_code_key(login_type: LoginType, login_identifier: &str) -> String {
    format!(
        "{}{}:{}",
        LOGIN_VERIFICATION_CODE_PREFIX,
        login_type.as_str(),
        login_identifier.trim()
    )
}
