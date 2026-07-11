use crate::error::AppError;
use crate::state::AppState;
use crate::tencent_cloud::tencent_live_license;
use crate::tencent_cloud::tencent_live_model::{
    DescribeLiveStreamStateReq, DescribeLiveStreamStateResp, LiveUrlsQuery, LiveUrlsResp,
};
use crate::tencent_cloud::tencent_live_service;
use crate::tencent_cloud::tencent_live_signer::build_live_authorization;
use axum::extract::{Query, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::{IntoResponse, Response};
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

/// 代理 Web 播放器 License，避免真实 URL 和 Key 出现在前端代码中。
pub async fn live_license(State(state): State<AppState>) -> Result<Response, AppError> {
    let config = state
        .tencent_live_license_config
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("腾讯云播放器 License 未配置".to_string()))?;
    let license = tencent_live_license::fetch_live_license(&state.http, config).await?;

    Ok((
        [
            (CONTENT_TYPE, license.content_type),
            (CACHE_CONTROL, "private, no-store".to_string()),
        ],
        license.body,
    )
        .into_response())
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
