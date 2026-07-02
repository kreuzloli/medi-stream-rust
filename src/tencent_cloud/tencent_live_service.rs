use crate::{
    common::constants::{
        http::{
            CONTENT_TYPE_JSON_UTF8, HEADER_AUTHORIZATION, HEADER_CONTENT_TYPE, HEADER_HOST,
            HEADER_X_TC_ACTION, HEADER_X_TC_TIMESTAMP, HEADER_X_TC_VERSION,
        },
        tencent_cloud::{
            ACTION_DESCRIBE_LIVE_STREAM_STATE, TENCENT_CLOUD_LIVE_SERVICE_NAME,
            TENCENT_LIVE_ENDPOINT, TENCENT_LIVE_HOST, TENCENT_LIVE_VERSION,
        },
    },
    error::AppError,
    state::AppState,
    tencent_cloud::{
        tencent_live_model::{
            DescribeLiveStreamStateReq, DescribeLiveStreamStateResp, LiveUrlsResp,
        },
        tencent_live_url_generator,
    },
};

pub async fn describe_live_stream_state(
    state: &AppState,
    req: &DescribeLiveStreamStateReq,
    authorization: &str,
    timestamp: i64,
) -> Result<DescribeLiveStreamStateResp, AppError> {
    let request = state
        .http
        .raw()
        .post(TENCENT_LIVE_ENDPOINT)
        .header(HEADER_AUTHORIZATION, authorization)
        .header(HEADER_CONTENT_TYPE, CONTENT_TYPE_JSON_UTF8)
        .header(HEADER_HOST, TENCENT_LIVE_HOST)
        .header(HEADER_X_TC_ACTION, ACTION_DESCRIBE_LIVE_STREAM_STATE)
        .header(HEADER_X_TC_VERSION, TENCENT_LIVE_VERSION)
        .header(HEADER_X_TC_TIMESTAMP, timestamp.to_string())
        .json(req);

    let resp = state
        .http
        .send_json::<DescribeLiveStreamStateResp>(TENCENT_CLOUD_LIVE_SERVICE_NAME, request)
        .await?;

    Ok(resp)
}

pub fn generate_live_urls(
    state: &AppState,
    stream_name: &str,
    ttl_seconds: Option<i64>,
    transcode_template: Option<&str>,
    now_epoch_seconds: i64,
) -> Result<LiveUrlsResp, AppError> {
    let config = state
        .tencent_live_url_config
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("腾讯云直播URL配置未配置".to_string()))?;

    tencent_live_url_generator::build_live_urls(
        config,
        stream_name,
        ttl_seconds,
        transcode_template,
        now_epoch_seconds,
    )
}
