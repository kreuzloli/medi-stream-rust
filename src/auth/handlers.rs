use crate::error::AppError;
use crate::state::AppState;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct LoginReq {
    username: String,
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
    // 这里保持 Java demo 的写死账号。后续要接真实用户表时，只需要替换这段校验逻辑。
    if req.username != "admin" || req.password != "123456" {
        return Err(AppError::Unauthorized("用户名或密码错误".to_string()));
    }

    let token = state
        .jwt
        .generate_token(&req.username, vec!["ADMIN".to_string()], Some(1))?;
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
