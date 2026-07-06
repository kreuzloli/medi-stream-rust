use crate::common::constants::wechat::{
    WECHAT_ACCESS_TOKEN_PATH, WECHAT_API_BASE_URL, WECHAT_CLIENT_CREDENTIAL, WECHAT_SERVICE_NAME,
};
use crate::error::AppError;
use crate::state::AppState;
use crate::wechat::wechat_model::WechatAccessTokenResp;
use sha1::{Digest, Sha1};

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
