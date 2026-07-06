use serde::Deserialize;

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
