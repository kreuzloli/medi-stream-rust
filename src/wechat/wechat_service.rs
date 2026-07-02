use crate::error::AppError;
use crate::state::AppState;
use crate::wechat::wechat_model::WechatAccessTokenResp;

pub async fn fetch_wechat_access_token(
    state: &AppState,
    app_id: &str,
    app_secret: &str,
) -> Result<WechatAccessTokenResp, AppError> {
    let url = format!(
        "https://api.weixin.qq.com/cgi-bin/token?grant_type=client_credential&appid={}&secret={}",
        app_id,
        app_secret
    );

    let resp = state
        .http
        .get_json::<WechatAccessTokenResp>("wechat", &url)
        .await?;
    Ok(resp)
}
