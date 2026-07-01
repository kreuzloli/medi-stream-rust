use crate::account::model::{AccountPageQuery, UserInfo};
use crate::account::{cache, repository};
use crate::common::Page;
use crate::error::AppError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::Json;

pub async fn create_account(
    headers: HeaderMap,
    State(mut state): State<AppState>,
    Json(req): Json<UserInfo>,
) -> Result<Json<UserInfo>, AppError> {
    // /account 接口需要登录。这里手动校验 token，逻辑更直观。
    state.jwt.require_headers(&headers)?;

    let id = repository::insert_user(&state.db, &req).await?;
    // 插入后重新查库，拿到数据库生成的 id、created_at、updated_at。
    let user = repository::find_user_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;
    cache::cache_user(&mut state, &user).await?;
    Ok(Json(user))
}

pub async fn get_account(
    headers: HeaderMap,
    State(mut state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Option<UserInfo>>, AppError> {
    state.jwt.require_headers(&headers)?;

    // Redis 里存的是 JSON 字符串；命中且反序列化成功就直接返回。
    if let Some(user) = cache::get_user(&mut state, id).await? {
        return Ok(Json(Some(user)));
    }

    let user = repository::find_user_by_id(&state.db, id).await?;
    if let Some(user) = &user {
        cache::cache_user(&mut state, user).await?;
    }
    Ok(Json(user))
}

pub async fn update_account(
    headers: HeaderMap,
    State(mut state): State<AppState>,
    Path(id): Path<u64>,
    Json(req): Json<UserInfo>,
) -> Result<Json<bool>, AppError> {
    state.jwt.require_headers(&headers)?;

    let ok = repository::update_user(&state.db, id, &req).await?;
    cache::delete_user_cache(&mut state, id).await?;
    Ok(Json(ok))
}

pub async fn delete_account(
    headers: HeaderMap,
    State(mut state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<bool>, AppError> {
    state.jwt.require_headers(&headers)?;

    let ok = repository::logical_delete_user(&state.db, id).await?;
    cache::delete_user_cache(&mut state, id).await?;
    Ok(Json(ok))
}

pub async fn page_accounts(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(query): Query<AccountPageQuery>,
) -> Result<Json<Page<UserInfo>>, AppError> {
    state.jwt.require_headers(&headers)?;

    Ok(Json(repository::page_users(&state.db, query).await?))
}
