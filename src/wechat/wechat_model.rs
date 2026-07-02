use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WechatAccessTokenResp {
    pub access_token: Option<String>,
    pub expires_in: Option<i64>,
    // 微信错误返回一般会有 errcode / errmsg。
    pub errcode: Option<i64>,
    pub errmsg: Option<String>,
}
