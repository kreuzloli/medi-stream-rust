use crate::account::account_service;
use crate::state::AppState;
use crate::wechat::wechat_model::{WechatCheckSignatureQuery, WechatOAuthCallbackQuery};
use crate::wechat::wechat_service;
use crate::{error::AppError, wechat::wechat_model::WechatOAuthAuthorizeQuery};
use axum::response::Redirect;
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

/// 微信 H5 OAuth 授权入口。
///
/// 前端没有 JWT 时访问：
/// GET /wechat/oauth/authorize?redirect=/wechat-live-play
///
/// 后端不在这里识别用户，只负责跳转到微信授权页。
pub async fn oauth_authorize(
    State(state): State<AppState>,
    Query(query): Query<WechatOAuthAuthorizeQuery>,
) -> Result<Redirect, AppError> {
    tracing::info!(
        redirect = %query.redirect,
        "wechat oauth_authorize started"
    );
    let authorize_url = wechat_service::build_wechat_oauth_authorize_url(&state, &query.redirect)?;
    tracing::info!("wechat oauth_authorize redirect to wechat");
    Ok(Redirect::temporary(&authorize_url))
}

/// 微信 H5 OAuth 回调。

///

/// 微信回调：

/// GET /wechat/oauth/callback?code=xxx&state=xxx

///

/// 处理流程：

/// 1. 用 code 换 openId。

/// 2. 根据 openId 查/建用户。

/// 3. 签发系统 JWT。

/// 4. 302 跳回前端 H5。

pub async fn oauth_callback(
    State(mut state): State<AppState>,
    Query(query): Query<WechatOAuthCallbackQuery>,
) -> Result<Redirect, AppError> {
    tracing::info!(
        code = query.code,
        state = %query.state,
        "wechat oauth_callback started"
    );
    let oauth_resp = wechat_service::fetch_wechat_oauth_access_token(&state, &query.code).await?;
    let (open_id, union_id) = wechat_service::parse_wechat_oauth_open_id(oauth_resp)?;
    let token =
        account_service::login_or_create_by_wechat(&mut state, &open_id, union_id.as_deref())
            .await?;
    let redirect_path = urlencoding::decode(&query.state)
        .map_err(|err| AppError::BadRequest(format!("微信 OAuth state 不合法: {err}")))?
        .to_string();

    let redirect_url = wechat_service::build_web_redirect_url(&state, &redirect_path, &token);
    Ok(Redirect::temporary(&redirect_url))
}
