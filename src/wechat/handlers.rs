use crate::error::AppError;
use crate::state::AppState;
use crate::wechat::wechat_model::WechatCheckSignatureQuery;
use crate::wechat::wechat_service;
use axum::{
    extract::{Query, State},
    Json,
};

/// 微信服务器配置校验接口。
///
/// 微信后台配置 URL 时，会发一个 GET 请求过来：
///
/// GET /wechat/callback?signature=xxx&timestamp=xxx&nonce=xxx&echostr=xxx
///
/// 如果签名正确，必须原样返回 echostr。
/// 不能返回 JSON。
pub async fn check_signature(
    State(state): State<AppState>,
    Query(query): Query<WechatCheckSignatureQuery>,
) -> Result<String, AppError> {
    let token = state
        .wechat_token
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::BadRequest("微信服务器校验 Token 未配置".to_string()))?;

    let is_valid =
        wechat_service::check_signature(token, &query.signature, &query.timestamp, &query.nonce);

    if !is_valid {
        tracing::warn!(
            timestamp = %query.timestamp,
            nonce = %query.nonce,
            "wechat check_signature failed"
        );

        return Err(AppError::Unauthorized("微信服务器签名校验失败".to_string()));
    }

    tracing::info!(
        timestamp = %query.timestamp,
        nonce = %query.nonce,
        "wechat check_signature succeeded"
    );

    // 微信要求成功后原样返回 echostr。
    // 注意这里不是 Json，而是纯文本字符串。
    Ok(query.echostr)
}

pub async fn reload_access_token(
    State(mut state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let access_token = wechat_service::get_wechat_access_token(&mut state).await?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "access_token_length": access_token.len()
    })))
}
