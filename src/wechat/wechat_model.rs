use serde::{Deserialize, Serialize};

use crate::wechat::wechat_enum::WechatLoginStatusEnum;

#[derive(Debug, Deserialize)]
pub struct WechatAccessTokenResp {
    pub access_token: Option<String>,
    pub expires_in: Option<i64>,
    // 微信错误返回一般会有 errcode / errmsg。
    pub errcode: Option<i64>,
    pub errmsg: Option<String>,
}

/// 微信服务器配置校验 GET 参数。

///

/// 微信会请求：

/// GET /wechat/callback?signature=xxx&timestamp=xxx&nonce=xxx&echostr=xxx

#[derive(Debug, Deserialize)]

pub struct WechatCheckSignatureQuery {
    /// 微信加密签名。
    ///
    /// signature = sha1(sort(token, timestamp, nonce).join(""))
    pub signature: String,

    /// 时间戳。
    pub timestamp: String,

    /// 随机字符串。
    pub nonce: String,

    /// 微信要求校验通过后原样返回的字符串。
    pub echostr: String,
}
/// 前端跳转到后端微信 OAuth 入口时的 query。
///
/// 示例：
/// /wechat/oauth/authorize?redirect=/wechat-live-play
#[derive(Debug, Deserialize)]
pub struct WechatOAuthAuthorizeQuery {
    /// 授权完成后要回到的前端 hash 路由。
    pub redirect: String,
}

/// 微信 OAuth callback query。
///
/// 微信会回调：
/// /wechat/oauth/callback?code=xxx&state=xxx
#[derive(Debug, Deserialize)]
pub struct WechatOAuthCallbackQuery {
    /// 微信返回的一次性 code。
    pub code: String,

    /// 后端 authorize 阶段传给微信的 state。
    ///
    /// 当前用它保存前端 redirect path。
    pub state: String,
}

/// 网页授权 code 换 openId 的响应。
///
/// 注意：这里的 access_token 是“网页授权 access_token”，
/// 不是公众号全局 access_token。
#[derive(Debug, Deserialize)]
pub struct WechatOAuthAccessTokenResp {
    pub access_token: Option<String>,
    pub expires_in: Option<i64>,
    pub refresh_token: Option<String>,
    pub openid: Option<String>,
    pub scope: Option<String>,
    pub unionid: Option<String>,

    /// 微信错误返回。
    pub errcode: Option<i64>,
    pub errmsg: Option<String>,
}

/// 获取WeChat二维码
#[derive(Serialize)]
pub struct WechatQrResponse {
    pub session_id: String,
    pub qr_url: String,
}

/// WeChat 登录状态
#[derive(Serialize)]
pub struct WechatLoginStatusResponse {
    pub status: WechatLoginStatusEnum,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub register_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WechatLoginSession {
    /// 前端轮询ID
    pub session_id: String,
    /// 状态枚举
    pub status: WechatLoginStatusEnum,
    /// 微信openid
    pub openid: Option<String>,
    /// 微信unionid
    pub unionid: Option<String>,
    /// 已登录账号
    pub account_id: Option<u64>,
    /// 未注册流程token
    pub register_token: Option<String>,
}
