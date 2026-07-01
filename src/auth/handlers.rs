use crate::account::model::LoginType;
use crate::account::{repository, service};
use crate::error::AppError;
use crate::state::AppState;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginReq {
    login_type: LoginType,
    login_identifier: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResp {
    token: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginReq>,
) -> Result<Json<LoginResp>, AppError> {
    // 不区分“账号不存在”和“密码错误”，避免登录接口泄露可枚举的账号信息。
    let login_account =
        repository::find_login_for_auth(&state.db, req.login_type, &req.login_identifier)
            .await?
            .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".to_string()))?;

    let Some(password_hash) = &login_account.password_hash else {
        return Err(AppError::Unauthorized(
            "当前登录方式不支持密码登录".to_string(),
        ));
    };
    if !service::verify_password(&req.password, password_hash)? {
        return Err(AppError::Unauthorized("用户名或密码错误".to_string()));
    }

    let token = state.jwt.generate_token(
        &login_account.login_identifier,
        vec!["USER".to_string()],
        Some(login_account.user_id),
    )?;
    Ok(Json(LoginResp { token }))
}

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
