use crate::account::model::UserInfo;
use crate::error::AppError;
use crate::state::AppState;
use redis::AsyncCommands;

const USER_INFO_CACHE_PREFIX: &str = "user_info:";
const USER_CACHE_SECONDS: u64 = 10 * 60;

pub async fn get_user(state: &mut AppState, id: u64) -> Result<Option<UserInfo>, AppError> {
    if let Some(redis) = state.redis.as_mut() {
        let cached: Option<String> = redis.get(user_cache_key(id)).await?;
        if let Some(cached) = cached {
            if let Ok(user) = serde_json::from_str::<UserInfo>(&cached) {
                return Ok(Some(user));
            }
        }
    }
    Ok(None)
}

pub async fn cache_user(state: &mut AppState, user: &UserInfo) -> Result<(), AppError> {
    // let Some(id) = ... else 是 Rust 的模式匹配写法；没有 id 时直接跳过缓存。
    let Some(id) = user.id else {
        return Ok(());
    };
    if let Some(redis) = state.redis.as_mut() {
        let json = serde_json::to_string(user)?;
        let _: () = redis
            .set_ex(user_cache_key(id), json, USER_CACHE_SECONDS)
            .await?;
    }
    Ok(())
}

pub async fn delete_user_cache(state: &mut AppState, id: u64) -> Result<(), AppError> {
    if let Some(redis) = state.redis.as_mut() {
        let _: () = redis.del(user_cache_key(id)).await?;
    }
    Ok(())
}

fn user_cache_key(id: u64) -> String {
    format!("{USER_INFO_CACHE_PREFIX}{id}")
}
