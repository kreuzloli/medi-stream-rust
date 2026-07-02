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

/// 处理直播 URL 生成接口请求。
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
    tracing::info!(
        stream_name = %query.stream_name.trim(),
        has_transcode_template = query.transcode_template.as_deref().is_some_and(|value| !value.trim().is_empty()),
        "generate_live_urls succeeded"
    );

    Ok(Json(resp))
}

/// 处理腾讯云直播流状态查询接口请求。
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
    tracing::info!(
        app_name = %req.app_name,
        domain_name = %req.domain_name,
        stream_name = %req.stream_name,
        "describe_live_stream_state request signed"
    );
    let resp =
        tencent_live_service::describe_live_stream_state(&state, &req, &authorization, timestamp)
            .await?;

    Ok(Json(resp))
}
