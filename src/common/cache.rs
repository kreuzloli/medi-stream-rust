use crate::account::account_model::{AccountDetail, LoginType};
use crate::common::constants::cache::{
    ACCOUNT_CACHE_SECONDS, ACCOUNT_DETAIL_CACHE_PREFIX, LOGIN_VERIFICATION_CODE_PREFIX,
    TOKEN_CACHE_PREFIX,
};
use crate::error::AppError;
use crate::state::AppState;
use redis::AsyncCommands;

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

pub async fn delete_account_cache(state: &mut AppState, id: u64) -> Result<(), AppError> {
    if let Some(redis) = state.redis.as_mut() {
        let _: () = redis.del(account_cache_key(id)).await?;
    }
    Ok(())
}

pub async fn delete_token_cache(state: &mut AppState, token: &str) -> Result<(), AppError> {
    let key = token_cache_key(token);
    if let Some(redis) = state.redis.as_mut() {
        let _: () = redis.del(key).await?;
    }
    Ok(())
}

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

fn account_cache_key(id: u64) -> String {
    format!("{ACCOUNT_DETAIL_CACHE_PREFIX}{id}")
}

fn token_cache_key(token: &str) -> String {
    format!("{TOKEN_CACHE_PREFIX}{token}")
}

fn login_verification_code_key(login_type: LoginType, login_identifier: &str) -> String {
    format!(
        "{}{}:{}",
        LOGIN_VERIFICATION_CODE_PREFIX,
        login_type.as_str(),
        login_identifier.trim()
    )
}
