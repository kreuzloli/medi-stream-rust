use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WechatLoginStatusEnum {
    /// 等待扫码
    Waiting,
    /// 已扫码
    Scanned,
    /// 登录成功
    Success,
    /// 微信账号未注册，需要完善资料
    RegisterRequired,
    /// 二维码过期
    Expired,
}
