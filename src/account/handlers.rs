use crate::account::model::{
    AccountDetail, AccountPageQuery, CreateAccountReq, CreateLoginAccountReq, RegisterResp,
    UpdateUserProfileReq, UserLoginAccount, UserProfile,
};
use crate::account::{cache, repository, service};
use crate::common::Page;
use crate::error::AppError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::Json;

pub async fn register(
    State(mut state): State<AppState>,
    Json(req): Json<CreateAccountReq>,
) -> Result<Json<RegisterResp>, AppError> {
    tracing::info!("register req: {:?}", req);
    let account = service::create_account(&mut state, req).await?;
    tracing::info!("register account: {:?}", account);
    let uid = account
        .profile
        .id
        .ok_or_else(|| AppError::Internal("registered account has no id".to_string()))?;
    let token = state.jwt.generate_token(
        &service::account_token_subject(&account),
        vec!["USER".to_string()],
        Some(uid),
    )?;
    tracing::info!("register token: {:?}", token);
    Ok(Json(RegisterResp { token }))
}

pub async fn get_account(
    headers: HeaderMap,
    State(mut state): State<AppState>,
) -> Result<Json<Option<AccountDetail>>, AppError> {
    let claims = state.jwt.require_headers(&headers)?;
    let id = service::require_claim_user_id(&claims)?;
    tracing::info!("get_account for user_id: {}", id);
    let account = match cache::get_account(&mut state, id).await? {
        Some(account) => {
            tracing::info!("get_account found account in cache: {:?}", account);
            Some(account)
        }
        None => {
            let account = repository::find_account_detail_by_id(&state.db, id).await?;
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

    Ok(Json(service::update_profile(&mut state, id, req).await?))
}

pub async fn delete_account(
    headers: HeaderMap,
    State(mut state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<bool>, AppError> {
    state.jwt.require_headers(&headers)?;

    Ok(Json(service::delete_account(&mut state, id).await?))
}

pub async fn page_accounts(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(query): Query<AccountPageQuery>,
) -> Result<Json<Page<UserProfile>>, AppError> {
    state.jwt.require_headers(&headers)?;

    Ok(Json(
        repository::page_user_profiles(&state.db, query).await?,
    ))
}

pub async fn bind_account(
    headers: HeaderMap,
    State(mut state): State<AppState>,
    Json(req): Json<CreateLoginAccountReq>,
) -> Result<Json<UserLoginAccount>, AppError> {
    let claims = state.jwt.require_headers(&headers)?;
    let user_id = service::require_claim_user_id(&claims)?;
    tracing::info!("bind_account for user_id: {}", user_id);
    Ok(Json(
        service::bind_account(&mut state, user_id, req).await?,
    ))
}

pub async fn unbind_account(
    headers: HeaderMap,
    State(mut state): State<AppState>,
    Path(login_id): Path<u64>,
) -> Result<Json<bool>, AppError> {
    let claims = state.jwt.require_headers(&headers)?;
    let user_id = service::require_claim_user_id(&claims)?;
    tracing::info!(
        "unbind_account for user_id: {}, login_id: {}",
        user_id,
        login_id
    );
    Ok(Json(
        service::unbind_account(&mut state, user_id, login_id).await?,
    ))
}
