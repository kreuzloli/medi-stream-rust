use crate::account::account_model::{CreateAccountReq, LoginType, RegisterResp};
use crate::account::{account_repository, account_service};
use crate::common::cache;
use crate::common::constants::auth::ROLE_USER;
use crate::error::AppError;
use crate::state::AppState;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginReq {
    login_type: LoginType,
    login_identifier: Option<String>,
    password: Option<String>,
    third_party_union_id: Option<String>,
    verification_code: Option<String>,
}

/// 处理登录请求，按登录方式校验凭证并签发 JWT。
pub async fn login(
    State(mut state): State<AppState>,
    Json(req): Json<LoginReq>,
) -> Result<Json<RegisterResp>, AppError> {
    tracing::info!(login_type = ?req.login_type, "login request received");
    let login_account = match req.login_type {
        LoginType::Email => {
            let login_identifier = account_service::require_login_identifier(
                req.login_type,
                req.login_identifier.as_deref(),
            )?;
            let login_account = account_repository::find_login_for_auth(
                &state.db,
                req.login_type,
                &login_identifier,
            )
            .await?
            .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".to_string()))?;
            account_service::validate_verified_login_account(
                req.login_type,
                login_account.is_verified,
            )?;
            let password = account_service::require_login_password(req.password.as_deref())?;
            let Some(password_hash) = &login_account.password_hash else {
                return Err(AppError::Unauthorized("用户名或密码错误".to_string()));
            };
            if !account_service::verify_password(&password, password_hash)? {
                return Err(AppError::Unauthorized("用户名或密码错误".to_string()));
            }
            login_account
        }
        LoginType::Phone => {
            let login_identifier = account_service::require_login_identifier(
                req.login_type,
                req.login_identifier.as_deref(),
            )?;
            let verification_code =
                account_service::require_login_verification_code(req.verification_code.as_deref())?;
            let login_account = account_repository::find_login_for_auth(
                &state.db,
                req.login_type,
                &login_identifier,
            )
            .await?
            .ok_or_else(|| AppError::Unauthorized("手机号或验证码错误".to_string()))?;
            account_service::validate_verified_login_account(
                req.login_type,
                login_account.is_verified,
            )?;
            cache::verify_login_verification_code(
                &mut state,
                req.login_type,
                &login_identifier,
                &verification_code,
            )
            .await?;
            login_account
        }
        LoginType::Wechat | LoginType::Github => {
            let third_party_union_id =
                account_service::require_third_party_union_id(req.third_party_union_id.as_deref())?;
            account_repository::find_login_for_auth_by_union_id(
                &state.db,
                req.login_type,
                &third_party_union_id,
            )
            .await?
            .ok_or_else(|| AppError::Unauthorized("第三方账号不存在".to_string()))?
        }
    };

    let account = account_repository::find_account_detail_by_id(&state.db, login_account.user_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("登录账户不可用".to_string()))?;
    let uid = account
        .profile
        .id
        .ok_or_else(|| AppError::Internal("login account has no user id".to_string()))?;
    let token = state.jwt.generate_token(
        &account_service::account_token_subject(&account),
        vec![ROLE_USER.to_string()],
        Some(uid),
    )?;
    account_repository::touch_last_login(
        &state.db,
        login_account.user_id,
        &login_account.login_identifier,
    )
    .await?;
    cache::cache_token(&mut state, &account, &token).await?;
    tracing::info!(
        user_id = uid,
        login_type = ?req.login_type,
        "login succeeded"
    );
    Ok(Json(RegisterResp { token }))
}

/// 返回当前 JWT 中的用户身份和角色信息。
pub async fn me(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    // HeaderMap 是当前请求的所有 header；require_headers 会校验 Bearer token。
    let claims = state.jwt.require_headers(&headers)?;
    Ok(Json(json!({
        "ok": true,
        "username": claims.sub,
        "roles": claims.roles,
        "uid": claims.uid
    })))
}

/// 注销当前 token，并删除 token 缓存。
pub async fn logout(
    headers: HeaderMap,
    State(mut state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = state.jwt.require_headers(&headers)?;
    let user_id = account_service::require_claim_user_id(&claims)?;
    let token = state.jwt.get_token_from_headers(&headers)?;
    cache::delete_token_cache(&mut state, &token).await?;
    tracing::info!(user_id, "logout succeeded");
    Ok(Json(json!({ "ok": true })))
}

/// 处理注册请求，创建账号后立即签发 JWT。
pub async fn register(
    State(mut state): State<AppState>,
    Json(req): Json<CreateAccountReq>,
) -> Result<Json<RegisterResp>, AppError> {
    tracing::info!(
        has_legacy_login = req.login_type.is_some(),
        login_account_count = req.login_accounts.len(),
        "register request received"
    );
    let account = account_service::create_account(&mut state, req).await?;
    let uid = account
        .profile
        .id
        .ok_or_else(|| AppError::Internal("registered account has no id".to_string()))?;
    let token = state.jwt.generate_token(
        &account_service::account_token_subject(&account),
        vec![ROLE_USER.to_string()],
        Some(uid),
    )?;
    cache::cache_token(&mut state, &account, &token).await?;
    tracing::info!(user_id = uid, "register succeeded");
    Ok(Json(RegisterResp { token }))
}
