use crate::common::cache;
use crate::common::constants::wechat::{
    WECHAT_ACCESS_TOKEN_PATH, WECHAT_API_BASE_URL, WECHAT_AUTHORIZATION_CODE,
    WECHAT_CLIENT_CREDENTIAL, WECHAT_OAUTH_ACCESS_TOKEN_PATH, WECHAT_OAUTH_AUTHORIZE_URL,
    WECHAT_OAUTH_CALLBACK_PATH, WECHAT_OAUTH_SCOPE_BASE, WECHAT_QRCODE_CALLBACK_PATH,
    WECHAT_SERVICE_NAME, WECHAT_SUCCESS_ERRCODE,
};
use crate::error::AppError;
use crate::state::AppState;
use crate::wechat::wechat_cache;
use crate::wechat::wechat_enum::WechatLoginStatusEnum;
use crate::wechat::wechat_model::{
    WechatAccessTokenResp, WechatLoginSession, WechatLoginStatusResponse,
    WechatOAuthAccessTokenResp, WechatQrResponse,
};
use sha1::{Digest, Sha1};

/// 给业务层使用：获取微信 access_token。
///
/// 逻辑：
/// 1. 优先从 Redis 读取。
/// 2. Redis 没有，再请求微信。
/// 3. 请求成功后写入 Redis。
///
/// 注意：
/// 这个方法是后台内部用的，不要暴露成 API。
pub async fn get_wechat_access_token(state: &mut AppState) -> Result<String, AppError> {
    let app_id = state
        .wechat_app_id
        .clone()
        .ok_or_else(|| AppError::Internal("WECHAT_APP_ID 未配置".to_string()))?;
    let app_secret = state
        .wechat_app_secret
        .clone()
        .ok_or_else(|| AppError::Internal("WECHAT_APP_SECRET 未配置".to_string()))?;
    let expire_seconds = state
        .wechat_access_token_expire_seconds
        .unwrap_or(7200)
        .saturating_sub(100)
        .max(60) as u64;

    if let Some(cached_token) = cache::get_wechat_access_token(state, &app_id).await? {
        return Ok(cached_token);
    }
    let resp = fetch_wechat_access_token(state, &app_id, &app_secret).await?;
    // let access_token = parse_wechat_access_token(resp)?;
    if let Some(errcode) = resp.errcode {
        if errcode != WECHAT_SUCCESS_ERRCODE {
            return Err(AppError::ExternalApi {
                service: WECHAT_SERVICE_NAME.to_string(),
                status: 200,
                body: format!(
                    "wechat errcode={}, errmsg={}",
                    errcode,
                    resp.errmsg
                        .unwrap_or_else(|| "unknown wechat error".to_string())
                ),
            });
        }
    }
    let access_token = resp
        .access_token
        .filter(|token| !token.trim().is_empty())
        .ok_or_else(|| AppError::ExternalApi {
            service: WECHAT_SERVICE_NAME.to_string(),
            status: 200,
            body: "wechat response missing access_token".to_string(),
        })?;
    cache::set_wechat_access_token(state, &app_id, &access_token, expire_seconds).await?;
    Ok(access_token)
}

/// 调用外部服务并返回解析后的响应。
pub async fn fetch_wechat_access_token(
    state: &AppState,
    app_id: &str,
    app_secret: &str,
) -> Result<WechatAccessTokenResp, AppError> {
    tracing::info!(app_id = %app_id, "fetch_wechat_access_token request started");
    let url = format!(
        "{}{}?grant_type={}&appid={}&secret={}",
        WECHAT_API_BASE_URL, WECHAT_ACCESS_TOKEN_PATH, WECHAT_CLIENT_CREDENTIAL, app_id, app_secret
    );

    let resp = state
        .http
        .get_json::<WechatAccessTokenResp>(WECHAT_SERVICE_NAME, &url)
        .await?;

    tracing::info!(app_id = %app_id, "fetch_wechat_access_token request finished");

    Ok(resp)
}

/// 生成微信公众号服务器推送校验签名。
///
/// 明文模式 / 初次接入校验用这个：
/// signature = sha1(sort(token, timestamp, nonce).join(""))
pub fn build_signature(token: &str, timestamp: &str, nonce: &str) -> String {
    build_sha1_signature(&[token, timestamp, nonce])
}

/// 校验微信公众号服务器推送签名。
///
/// 返回 true：说明这个请求大概率来自微信服务器。
/// 返回 false：说明签名不匹配，应该拒绝。
pub fn check_signature(token: &str, signature: &str, timestamp: &str, nonce: &str) -> bool {
    let expected = build_signature(token, timestamp, nonce);

    // 微信一般给的是小写 hex。
    // 这里用 eq_ignore_ascii_case，避免大小写导致误杀。
    expected.eq_ignore_ascii_case(signature)
}

/// 生成加密消息模式下的 msg_signature。
///
/// 你上传的多语言示例里，很多是这个逻辑：
/// msg_signature = sha1(sort(token, timestamp, nonce, encrypt).join(""))
///
/// 注意：
/// 这个只负责“验签”，不负责 AES 解密。
/// 后面如果要处理安全模式消息，先从 XML 里取出 Encrypt，再调用这个。
pub fn build_msg_signature(token: &str, timestamp: &str, nonce: &str, encrypt: &str) -> String {
    build_sha1_signature(&[token, timestamp, nonce, encrypt])
}

/// 校验加密消息模式下的 msg_signature。
pub fn check_msg_signature(
    token: &str,
    msg_signature: &str,
    timestamp: &str,
    nonce: &str,
    encrypt: &str,
) -> bool {
    let expected = build_msg_signature(token, timestamp, nonce, encrypt);
    expected.eq_ignore_ascii_case(msg_signature)
}

/// 公共 SHA1 签名逻辑。
///
/// 微信的规则就是：
/// 1. 把参数按字典序排序。
/// 2. 拼接成一个字符串。
/// 3. 对拼接结果做 SHA1。
fn build_sha1_signature(parts: &[&str]) -> String {
    let mut sorted_parts = parts.to_vec();

    // 字典序排序。
    sorted_parts.sort_unstable();

    // 拼接排序后的字符串。
    let raw = sorted_parts.concat();

    // 做 SHA1。
    let mut hasher = Sha1::new();
    hasher.update(raw.as_bytes());

    // 转成小写 16 进制字符串。
    format!("{:x}", hasher.finalize())
}

/// 构建微信网页授权地址。
///
/// 前端访问：
/// /wechat/oauth/authorize?redirect=/wechat-live-play
///
/// 后端会重定向到微信：
/// https://open.weixin.qq.com/connect/oauth2/authorize...
pub fn build_wechat_oauth_authorize_url(
    state: &AppState,
    redirect_path: &str,
) -> Result<String, AppError> {
    let app_id = state
        .wechat_app_id
        .as_deref()
        .map(str::trim)
        .filter(|val| !val.is_empty())
        .ok_or_else(|| AppError::Internal("WECHAT_APP_ID 未配置".to_string()))?;
    let callback_base_url = state
        .wechat_oauth_callback_base_url
        .as_deref()
        .unwrap_or(&state.web_base_url)
        .trim_end_matches('/');

    let callback_url = format!("{callback_base_url}{WECHAT_OAUTH_CALLBACK_PATH}");
    // state 暂时保存前端路由，例如 /wechat-live-play。
    // 这里做 URL 编码，避免特殊字符破坏微信授权 URL。
    let encoded_callback = urlencoding::encode(&callback_url);
    let encoded_state = urlencoding::encode(redirect_path);

    let url = format!(
        "{WECHAT_OAUTH_AUTHORIZE_URL}?appid={app_id}&redirect_uri={encoded_callback}&response_type=code&scope={WECHAT_OAUTH_SCOPE_BASE}&state={encoded_state}#wechat_redirect"
    );

    tracing::info!(
        app_id = %app_id,
        callback_url = %callback_url,
        redirect_path = %redirect_path,
        "build_wechat_oauth_authorize_url finished"
    );
    Ok(url)
}

/// 使用微信 OAuth code 换取 openId。
///
/// 注意：
/// 这个接口返回的是网页授权 access_token 和 openId，
/// 不是公众号全局 access_token。
pub async fn fetch_wechat_oauth_access_token(
    state: &AppState,
    code: &str,
) -> Result<WechatOAuthAccessTokenResp, AppError> {
    let app_id = state
        .wechat_app_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::Internal("WECHAT_APP_ID 未配置".to_string()))?;

    let app_secret = state
        .wechat_app_secret
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::Internal("WECHAT_APP_SECRET 未配置".to_string()))?;

    tracing::info!(
        app_id = %app_id,
        code_len = code.len(),
        "fetch_wechat_oauth_access_token request started"
    );
    let url = format!(
        "{}{}?appid={}&secret={}&code={}&grant_type={}",
        WECHAT_API_BASE_URL,
        WECHAT_OAUTH_ACCESS_TOKEN_PATH,
        app_id,
        app_secret,
        code,
        WECHAT_AUTHORIZATION_CODE
    );

    let resp = state
        .http
        .get_json::<WechatOAuthAccessTokenResp>(WECHAT_SERVICE_NAME, &url)
        .await?;

    tracing::info!(
        app_id = %app_id,
        open_id = resp.openid,
        union_id = resp.unionid,
        "fetch_wechat_oauth_access_token request finished"
    );
    Ok(resp)
}

/// 从微信 OAuth 响应中解析 openId 和 unionId。

pub fn parse_wechat_oauth_open_id(
    resp: WechatOAuthAccessTokenResp,
) -> Result<(String, Option<String>), AppError> {
    if let Some(errcode) = resp.errcode {
        if errcode != WECHAT_SUCCESS_ERRCODE {
            return Err(AppError::ExternalApi {
                service: WECHAT_SERVICE_NAME.to_string(),
                status: 200,
                body: format!(
                    "wechat oauth errcode={}, errmsg={}",
                    errcode,
                    resp.errmsg
                        .unwrap_or_else(|| "unknown wechat oauth error".to_string())
                ),
            });
        }
    }
    let open_id = resp
        .openid
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError::ExternalApi {
            service: WECHAT_SERVICE_NAME.to_string(),
            status: 200,
            body: "wechat oauth response missing openid".to_string(),
        })?;
    tracing::info!(
        open_id = open_id,
        union_id = resp.unionid,
        "parse_wechat_oauth_open_id finished"
    );
    Ok((open_id, resp.unionid))
}

/// 构建 OAuth 成功后的前端回跳地址。
///
/// 示例：
/// https://web.example.com/#/wechat-live-play?token=xxx
pub fn build_web_redirect_url(state: &AppState, redirect_path: &str, token: &str) -> String {
    let web_base_url = state.web_base_url.trim_end_matches('/');
    let normalized_path = if redirect_path.starts_with('/') {
        redirect_path.to_string()
    } else {
        format!("/{redirect_path}")
    };
    let encoded_token = urlencoding::encode(token);
    let url = format!("{web_base_url}/#{normalized_path}?token={encoded_token}");

    tracing::info!(
        redirect_path = %normalized_path,
        token_len = token.len(),
        "build_web_redirect_url finished"
    );
    url
}

/// 构建供 Web 页面渲染为二维码的微信公众号 OAuth 地址。
///
/// 这里只返回二维码承载的 OAuth 地址，不在服务端生成 PNG 或 Base64 图片。
pub fn build_wechat_qrcode_authorize_url(
    state: &AppState,
    session_id: &str,
) -> Result<String, AppError> {
    let app_id = state
        .wechat_app_id
        .as_deref()
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .ok_or_else(|| AppError::Internal("WECHAT_APP_ID 未配置".to_string()))?;
    let callback_base_url = state
        .wechat_oauth_callback_base_url
        .as_deref()
        .unwrap_or(&state.web_base_url)
        .trim_end_matches('/');
    let callback_url = format!("{callback_base_url}{WECHAT_QRCODE_CALLBACK_PATH}");
    let encoded_callback = urlencoding::encode(&callback_url);
    // 增加固定前缀，后续回调可以区分普通 H5 OAuth 和扫码登录。
    let qr_state = format!("qr_login:{session_id}");
    let encoded_state = urlencoding::encode(&qr_state);

    let authorize_url = format!(
        "{WECHAT_OAUTH_AUTHORIZE_URL}\
    ?appid={app_id}\
    &redirect_uri={encoded_callback}\
    &response_type=code\
    &scope={WECHAT_OAUTH_SCOPE_BASE}\
    &state={encoded_state}\
    #wechat_redirect"
    );
    tracing::debug!(
        session_id = %session_id,
        callback_url = %callback_url,
        "wechat qrcode authorize url built"
    );
    Ok(authorize_url)
}

/// 创建微信扫码登录会话，并返回前端渲染二维码所需的 OAuth 地址。
///
/// 会话必须成功写入 Redis 后才能返回，避免前端拿到无法轮询的二维码。
pub async fn create_qrcode(state: &AppState) -> Result<WechatQrResponse, AppError> {
    let session_id = uuid::Uuid::new_v4().to_string();
    // 先校验微信配置并构造地址，避免配置错误时产生无用 Redis 会话。
    let qr_url = build_wechat_qrcode_authorize_url(state, &session_id)?;
    let session = WechatLoginSession {
        session_id: session_id.clone(),
        status: WechatLoginStatusEnum::Waiting,
        openid: None,
        unionid: None,
        account_id: None,
        register_token: None,
    };
    wechat_cache::set_wechat_login_session(state, &session).await?;
    tracing::info!(
        session_id = %session_id,
        status = ?WechatLoginStatusEnum::Waiting,
        "wechat qrcode login session created"
    );

    Ok(WechatQrResponse { session_id, qr_url })
}

/// 查询微信扫码登录会话状态。
///
/// Redis key 不存在属于二维码正常过期，返回 EXPIRED，不转换成系统异常。
pub async fn get_status(
    state: &AppState,
    session_id: &str,
) -> Result<WechatLoginStatusResponse, AppError> {
    let Some(session) = wechat_cache::get_wechat_login_session(state, session_id).await? else {
        tracing::debug!(
            session_id = %session_id,
            "wechat qrcode login session expired"
        );
        return Ok(WechatLoginStatusResponse {
            status: WechatLoginStatusEnum::Expired,
            token: None,
            register_token: None,
        });
    };
    tracing::debug!(
        session_id = %session_id,
        status = ?session.status,
        has_register_token = session.register_token.is_some(),
        "wechat qrcode login status queried"
    );

    Ok(WechatLoginStatusResponse {
        status: session.status,
        token: None,
        register_token: session.register_token,
    })
}
