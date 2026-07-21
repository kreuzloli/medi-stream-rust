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
    /// 登录成功后签发的系统 JWT。
    #[serde(default)]
    pub token: Option<String>,
    /// 未注册用户完善资料时使用的一次性凭证。
    #[serde(default)]
    pub register_token: Option<String>,
}

/// 微信扫码后完善资料的请求。
///
/// openId、unionId 不允许由前端提交，服务端使用 registerToken 从 Redis 获取。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WechatQrcodeRegisterReq {
    /// 微信回调生成的一次性注册凭证。
    pub register_token: String,
    /// 姓名，必填。
    pub real_name: String,
    /// 昵称，必填。
    pub nickname: String,
    /// MEDICAL_WORKER 或 NON_MEDICAL_WORKER，必填。
    pub identity_type: String,
    /// 医疗从业者必填。
    pub hospital_id: Option<u64>,
    /// 医疗从业者必填。
    pub dept_id: Option<u64>,
    pub doctor_cert_no: Option<String>,
    pub id_card_no: Option<String>,
    /// 联系电话，不作为登录账号。
    pub mobile: Option<String>,
    /// 头像文件 ID。
    pub header_id: Option<u64>,
}

/// 微信扫码注册上下文。
///
/// 只保存在 Redis，不写入数据库；用户提交完善资料后删除。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WechatRegisterContext {
    /// 对应的二维码登录会话。
    pub session_id: String,
    /// 微信 OAuth 返回的 openId。
    pub openid: String,
    /// 微信 OAuth 返回的 unionId，公众号未绑定开放平台时可能为空。
    pub unionid: Option<String>,
}
