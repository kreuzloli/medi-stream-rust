use crate::error::AppError;
use crate::state::AppState;
use crate::tencent_cloud::tencent_live_model::{
    DescribeLiveStreamStateReq, DescribeLiveStreamStateResp, LiveUrlsQuery, LiveUrlsResp,
};
use crate::tencent_cloud::tencent_live_service;
use crate::tencent_cloud::tencent_live_signer::build_live_authorization;
use axum::extract::{Query, State};
use axum::Json;
use chrono::Utc;

pub async fn generate_live_urls(
    State(state): State<AppState>,
    Query(query): Query<LiveUrlsQuery>,
) -> Result<Json<LiveUrlsResp>, AppError> {
    let resp = tencent_live_service::generate_live_urls(
        &state,
        &query.stream_name,
        query.ttl_seconds,
        query.transcode_template.as_deref(),
        Utc::now().timestamp(),
    )?;

    Ok(Json(resp))
}

pub async fn describe_live_stream_state(
    State(state): State<AppState>,
    Json(req): Json<DescribeLiveStreamStateReq>,
) -> Result<Json<DescribeLiveStreamStateResp>, AppError> {
    req.validate()?;
    let credential = state
        .tencent_live_credential
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("腾讯云直播凭证未配置".to_string()))?;
    let timestamp = Utc::now().timestamp();
    let authorization = build_live_authorization(credential, timestamp, &req)?;
    let resp =
        tencent_live_service::describe_live_stream_state(&state, &req, &authorization, timestamp)
            .await?;

    Ok(Json(resp))
}
