use crate::common::constants::wechat::{
    WECHAT_ACCESS_TOKEN_PATH, WECHAT_API_BASE_URL, WECHAT_CLIENT_CREDENTIAL, WECHAT_SERVICE_NAME,
};
use crate::error::AppError;
use crate::state::AppState;
use crate::wechat::wechat_model::WechatAccessTokenResp;

pub async fn fetch_wechat_access_token(
    state: &AppState,
    app_id: &str,
    app_secret: &str,
) -> Result<WechatAccessTokenResp, AppError> {
    let url = format!(
        "{}{}?grant_type={}&appid={}&secret={}",
        WECHAT_API_BASE_URL, WECHAT_ACCESS_TOKEN_PATH, WECHAT_CLIENT_CREDENTIAL, app_id, app_secret
    );

    let resp = state
        .http
        .get_json::<WechatAccessTokenResp>(WECHAT_SERVICE_NAME, &url)
        .await?;

    Ok(resp)
}
