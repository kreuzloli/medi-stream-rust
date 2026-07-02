use crate::{
    error::AppError,
    state::AppState,
    tencent_cloud::tencent_live_model::{DescribeLiveStreamStateReq, DescribeLiveStreamStateResp},
};

pub async fn describe_live_stream_state(
    state: &AppState,
    req: &DescribeLiveStreamStateReq,
    authorization: &str,
    timestamp: i64,
) -> Result<DescribeLiveStreamStateResp, AppError> {
    let url = "https://live.tencentcloudapi.com";

    let request = state
        .http
        .raw()
        .post(url)
        .header("Authorization", authorization)
        .header("Content-Type", "application/json; charset=utf-8")
        .header("Host", "live.tencentcloudapi.com")
        .header("X-TC-Action", "DescribeLiveStreamState")
        .header("X-TC-Version", "2018-08-01")
        .header("X-TC-Timestamp", timestamp.to_string())
        .json(req);

    let resp = state
        .http
        .send_json::<DescribeLiveStreamStateResp>("tencent_cloud_live", request)
        .await?;

    Ok(resp)
}
