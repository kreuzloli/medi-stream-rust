use crate::account::account_model::{
    AccountDetail, CreateLoginAccountReq, UpdateUserProfileReq, UserLoginAccount,
};
use crate::account::{account_repository, account_service};
use crate::common::cache;
use crate::common::jwt::CurrentUser;
use crate::error::AppError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;

/// 查询当前 JWT 用户的账号详情，优先读取缓存。
pub async fn get_account(
    CurrentUser(claims): CurrentUser,
    State(mut state): State<AppState>,
) -> Result<Json<Option<AccountDetail>>, AppError> {
    let id = account_service::require_claim_user_id(&claims)?;
    tracing::info!(user_id = id, "get_account request received");
    let account = match cache::get_account(&mut state, id).await? {
        Some(account) => {
            tracing::info!(user_id = id, "get_account cache hit");
            Some(account)
        }
        None => {
            let account = account_repository::find_account_detail_by_id(&state.db, id).await?;
            match &account {
                Some(account) => {
                    tracing::info!(user_id = id, "get_account database hit");
                    cache::cache_account(&mut state, account).await?;
                }
                None => {
                    tracing::info!(user_id = id, "get_account not found");
                }
            }
            account
        }
    };
    Ok(Json(account))
}

/// 更新业务数据，并在目标不存在时返回 NotFound。
pub async fn update_account(
    CurrentUser(_claims): CurrentUser,
    State(mut state): State<AppState>,
    Path(id): Path<u64>,
    Json(req): Json<UpdateUserProfileReq>,
) -> Result<Json<bool>, AppError> {
    tracing::info!(user_id = id, "update_account request received");
    let updated = account_service::update_profile(&mut state, id, req).await?;
    tracing::info!(user_id = id, updated, "update_account finished");
    Ok(Json(updated))
}

/// 为当前 JWT 用户绑定新的登录方式。
pub async fn bind_account(
    CurrentUser(claims): CurrentUser,
    State(mut state): State<AppState>,
    Json(req): Json<CreateLoginAccountReq>,
) -> Result<Json<UserLoginAccount>, AppError> {
    let user_id = account_service::require_claim_user_id(&claims)?;
    tracing::info!(user_id, login_type = ?req.login_type, "bind_account request received");
    Ok(Json(
        account_service::bind_account(&mut state, user_id, req).await?,
    ))
}

/// 解绑当前 JWT 用户的一条登录方式。
pub async fn unbind_account(
    CurrentUser(claims): CurrentUser,
    State(mut state): State<AppState>,
    Path(login_id): Path<u64>,
) -> Result<Json<bool>, AppError> {
    let user_id = account_service::require_claim_user_id(&claims)?;
    tracing::info!(user_id, login_id, "unbind_account request received");
    let deleted = account_service::unbind_account(&mut state, user_id, login_id).await?;
    tracing::info!(user_id, login_id, deleted, "unbind_account finished");
    Ok(Json(deleted))
}
