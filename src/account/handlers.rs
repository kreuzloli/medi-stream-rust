use crate::account::account_model::{
    AccountDetail, CreateLoginAccountReq, UpdateUserProfileReq, UserLoginAccount,
};
use crate::account::{account_repository, account_service};
use crate::common::cache;
use crate::error::AppError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;

pub async fn get_account(
    headers: HeaderMap,
    State(mut state): State<AppState>,
) -> Result<Json<Option<AccountDetail>>, AppError> {
    let claims = state.jwt.require_headers(&headers)?;
    let id = account_service::require_claim_user_id(&claims)?;
    tracing::info!("get_account for user_id: {}", id);
    let account = match cache::get_account(&mut state, id).await? {
        Some(account) => {
            tracing::info!("get_account found account in cache: {:?}", account);
            Some(account)
        }
        None => {
            let account = account_repository::find_account_detail_by_id(&state.db, id).await?;
            match &account {
                Some(account) => {
                    tracing::info!("get_account found account in database: {:?}", account);
                    cache::cache_account(&mut state, account).await?;
                }
                None => {
                    tracing::info!("get_account did not find account for user_id: {}", id);
                }
            }
            account
        }
    };
    Ok(Json(account))
}

pub async fn update_account(
    headers: HeaderMap,
    State(mut state): State<AppState>,
    Path(id): Path<u64>,
    Json(req): Json<UpdateUserProfileReq>,
) -> Result<Json<bool>, AppError> {
    state.jwt.require_headers(&headers)?;

    Ok(Json(
        account_service::update_profile(&mut state, id, req).await?,
    ))
}

pub async fn bind_account(
    headers: HeaderMap,
    State(mut state): State<AppState>,
    Json(req): Json<CreateLoginAccountReq>,
) -> Result<Json<UserLoginAccount>, AppError> {
    let claims = state.jwt.require_headers(&headers)?;
    let user_id = account_service::require_claim_user_id(&claims)?;
    tracing::info!("bind_account for user_id: {}", user_id);
    Ok(Json(
        account_service::bind_account(&mut state, user_id, req).await?,
    ))
}

pub async fn unbind_account(
    headers: HeaderMap,
    State(mut state): State<AppState>,
    Path(login_id): Path<u64>,
) -> Result<Json<bool>, AppError> {
    let claims = state.jwt.require_headers(&headers)?;
    let user_id = account_service::require_claim_user_id(&claims)?;
    tracing::info!(
        "unbind_account for user_id: {}, login_id: {}",
        user_id,
        login_id
    );
    Ok(Json(
        account_service::unbind_account(&mut state, user_id, login_id).await?,
    ))
}
