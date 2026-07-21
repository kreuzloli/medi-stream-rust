use crate::account::account_model::RegisterResp;
use crate::account::account_service;
use crate::file::{file_service, file_storage};
use crate::state::AppState;
use crate::wechat::wechat_model::{
    WechatCheckSignatureQuery, WechatLoginStatusResponse, WechatOAuthCallbackQuery,
    WechatQrResponse, WechatQrcodeRegisterReq, WechatRegisterFileKind, WechatRegisterFileResp,
};
use crate::wechat::wechat_service;
use crate::{error::AppError, wechat::wechat_model::WechatOAuthAuthorizeQuery};
use axum::extract::{Multipart, Path};
use axum::response::{Html, Redirect};
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

/// 主动刷新或读取微信公众号全局 Access Token。
///
/// 响应只返回长度用于诊断，不向调用方暴露 Token 内容。
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
        code_len = query.code.len(),
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

/// 创建微信扫码登录会话。
///
/// 返回的 qr_url 由 Web 端渲染为二维码，session_id 用于后续轮询登录状态。
pub async fn create_qrcode(
    State(state): State<AppState>,
) -> Result<Json<WechatQrResponse>, AppError> {
    tracing::info!("wechat qrcode creation request received");
    let res = wechat_service::create_qrcode(&state).await?;
    tracing::info!(
        session_id = %res.session_id,
        "wechat qrcode creation request completed"
    );
    Ok(Json(res))
}

/// 查询微信扫码登录会话状态。
///
/// 二维码过期属于正常业务状态，由 service 返回 EXPIRED。
pub async fn get_status(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<WechatLoginStatusResponse>, AppError> {
    tracing::debug!(
        session_id = %session_id,
        "wechat qrcode status request received"
    );
    let result = wechat_service::get_status(&state, &session_id).await?;
    tracing::debug!(
        session_id = %session_id,
        status = ?result.status,
        "wechat qrcode status request completed"
    );
    Ok(Json(result))
}

/// 微信扫码登录 OAuth 回调。
///
/// 回调只确认微信身份。已有账号直接登录；新用户进入资料完善流程。
pub async fn qrcode_callback(
    State(mut state): State<AppState>,
    Query(query): Query<WechatOAuthCallbackQuery>,
) -> Result<Html<&'static str>, AppError> {
    tracing::info!(
        code_len = query.code.len(),
        state_len = query.state.len(),
        "wechat qrcode callback received"
    );
    let session_id =
        wechat_service::complete_qrcode_login(&mut state, &query.code, &query.state).await?;
    tracing::info!(
        session_id = %session_id,
        "wechat qrcode callback completed"
    );

    Ok(Html(
        r#"<!doctype html>
    <html lang="zh-CN">
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width,initial-scale=1">
        <title>微信身份确认成功</title>
        <style>
            body {
                margin: 0;
                min-height: 100vh;
                display: grid;
                place-items: center;
                background: #f5f7fb;
                color: #16234a;
                font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
            }
            main {
                width: min(84vw, 360px);
                padding: 40px 24px;
                text-align: center;
                background: #fff;
                border: 1px solid #e8edf5;
                border-radius: 16px;
                box-shadow: 0 12px 36px rgba(27, 45, 94, .08);
            }
            .icon {
                width: 56px;
                height: 56px;
                margin: 0 auto 20px;
                display: grid;
                place-items: center;
                color: #fff;
                font-size: 30px;
                border-radius: 50%;
                background: #2f6bff;
            }
            h1 { margin: 0 0 12px; font-size: 22px; }
            p { margin: 0; color: #687794; line-height: 1.7; }
        </style>
    </head>
    <body>
        <main>
            <div class="icon">✓</div>
            <h1>微信身份确认成功</h1>
            <p>请返回电脑浏览器继续登录或完善个人资料。</p>
        </main>
    </body>
    </html>"#,
    ))
}

/// 完善微信扫码用户资料并创建正式账号。
///
/// 该接口不要求 JWT；短期、一次性的 registerToken 用于证明
/// 当前注册请求已经通过微信 OAuth 身份验证。
pub async fn qrcode_register(
    State(mut state): State<AppState>,
    Json(req): Json<WechatQrcodeRegisterReq>,
) -> Result<Json<RegisterResp>, AppError> {
    tracing::info!(
        real_name = %req.real_name,
        nickname = %req.nickname,
        identity_type = %req.identity_type,
        hospital_id = ?req.hospital_id,
        dept_id = ?req.dept_id,
        mobile = ?req.mobile,
        header_id = ?req.header_id,
        doctor_cert_file_id = ?req.doctor_cert_file_id,
        id_card_front_file_id = ?req.id_card_front_file_id,
        id_card_back_file_id = ?req.id_card_back_file_id,
        doctor_cert_no = ?req.doctor_cert_no,
        id_card_no = ?req.id_card_no,
        "wechat qrcode registration request received"
    );

    let result = wechat_service::register_qrcode_account(&mut state, req).await?;

    tracing::info!("wechat qrcode registration request completed");
    Ok(Json(result))
}

/// 上传扫码注册阶段使用的头像或证件文件。
///
/// registerToken 将文件绑定到当前微信注册上下文，注册提交时只能引用这里上传的文件 ID。
pub async fn qrcode_upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<WechatRegisterFileResp>, AppError> {
    let mut register_token = None;
    let mut kind = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| AppError::BadRequest(format!("读取上传表单失败: {error}")))?
    {
        match field.name().unwrap_or_default() {
            "registerToken" => register_token = Some(field.text().await.map_err(bad_upload)?),
            "kind" => kind = Some(field.text().await.map_err(bad_upload)?),
            "file" => {
                // 流式 Multipart 的文件字段不能暂存后再读取，因此要求前端先提交注册凭证和用途。
                let register_token = register_token
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        AppError::BadRequest("registerToken 字段必须位于 file 字段之前".to_string())
                    })?;
                uuid::Uuid::parse_str(register_token)
                    .map_err(|_| AppError::Unauthorized("微信注册凭证无效或已过期".to_string()))?;
                let kind = kind
                    .as_deref()
                    .and_then(WechatRegisterFileKind::parse)
                    .ok_or_else(|| {
                        AppError::BadRequest("kind 字段必须位于 file 字段之前".to_string())
                    })?;
                let file_name = field.file_name().unwrap_or("upload.bin").to_string();
                let mime_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                validate_register_upload(kind, &mime_type)?;

                // 必须先确认 registerToken 仍有效，再向共享目录写文件。
                let mut context = crate::wechat::wechat_cache::get_wechat_register_context(
                    &state,
                    register_token,
                )
                .await?
                .ok_or_else(|| AppError::Unauthorized("微信注册凭证无效或已过期".to_string()))?;

                tracing::info!(
                    session_id = %context.session_id,
                    register_token = %register_token,
                    kind = kind.as_str(),
                    file_name = %file_name,
                    mime_type = %mime_type,
                    "wechat registration file upload received"
                );

                let stored_file =
                    file_storage::save_uploaded_file(field, &state.file_storage).await?;
                let file = file_service::create_uploaded_file_object(&state, stored_file).await?;
                let previous_file_id = context
                    .uploaded_files
                    .insert(kind.as_str().to_string(), file.id);

                if let Err(error) = crate::wechat::wechat_cache::set_wechat_register_context(
                    &state,
                    register_token,
                    &context,
                )
                .await
                {
                    tracing::error!(
                        session_id = %context.session_id,
                        file_id = file.id,
                        error = %error,
                        "wechat register context update failed, rollback uploaded file"
                    );
                    if let Err(cleanup_error) =
                        file_service::delete_file_object(&state, file.id).await
                    {
                        tracing::error!(
                            file_id = file.id,
                            error = %cleanup_error,
                            "wechat registration file rollback failed"
                        );
                    }
                    return Err(error);
                }

                // 同一用途重复上传时，缓存已指向新文件，旧的临时注册文件可以安全清理。
                if let Some(previous_file_id) = previous_file_id.filter(|id| *id != file.id) {
                    if let Err(error) =
                        file_service::delete_file_object(&state, previous_file_id).await
                    {
                        tracing::warn!(
                            session_id = %context.session_id,
                            kind = kind.as_str(),
                            previous_file_id,
                            error = %error,
                            "previous wechat registration file cleanup failed"
                        );
                    }
                }

                tracing::info!(
                    session_id = %context.session_id,
                    register_token = %register_token,
                    kind = kind.as_str(),
                    file_id = file.id,
                    file_name = %file.file_name,
                    file_url = %file.file_url,
                    mime_type = ?file.mime_type,
                    file_size = ?file.file_size,
                    sha256 = ?file.sha256,
                    "wechat registration file upload completed"
                );
                return Ok(Json(WechatRegisterFileResp {
                    file_id: file.id,
                    file_name: file.file_name,
                    kind: kind.as_str().to_string(),
                }));
            }
            _ => {}
        }
    }

    Err(AppError::BadRequest(
        "上传请求中缺少 file 文件字段".to_string(),
    ))
}

fn bad_upload(error: axum::extract::multipart::MultipartError) -> AppError {
    AppError::BadRequest(format!("读取上传文件失败: {error}"))
}

fn validate_register_upload(kind: WechatRegisterFileKind, mime_type: &str) -> Result<(), AppError> {
    let is_image = matches!(mime_type, "image/jpeg" | "image/png" | "image/webp");
    if kind == WechatRegisterFileKind::Avatar && !is_image {
        return Err(AppError::BadRequest(
            "头像只支持 JPG、PNG 或 WebP 图片".to_string(),
        ));
    }
    if !is_image && mime_type != "application/pdf" {
        return Err(AppError::BadRequest(
            "证件只支持 JPG、PNG、WebP 或 PDF".to_string(),
        ));
    }
    Ok(())
}
