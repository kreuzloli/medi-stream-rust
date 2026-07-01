use crate::account::model::AccountDetail;
use crate::error::AppError;
use crate::state::AppState;
use redis::AsyncCommands;

const ACCOUNT_DETAIL_CACHE_PREFIX: &str = "account_detail:";
const ACCOUNT_CACHE_SECONDS: u64 = 10 * 60;

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

pub async fn delete_account_cache(state: &mut AppState, id: u64) -> Result<(), AppError> {
    if let Some(redis) = state.redis.as_mut() {
        let _: () = redis.del(account_cache_key(id)).await?;
    }
    Ok(())
}

fn account_cache_key(id: u64) -> String {
    format!("{ACCOUNT_DETAIL_CACHE_PREFIX}{id}")
}
