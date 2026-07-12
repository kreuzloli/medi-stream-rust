pub mod handlers;
pub mod tencent_live_license;
pub mod tencent_live_model;
pub mod tencent_live_service;
pub mod tencent_live_signer;
pub mod tencent_live_url_generator;

use crate::common::constants::route;
use crate::state::AppState;
use axum::Router;
use axum::routing::{get, post};

/// 注册腾讯云直播 URL、状态查询和播放器 License 接口。
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(route::LIVE_URLS, get(handlers::generate_live_urls))
        .route(
            route::LIVE_STREAM_STATE,
            post(handlers::describe_live_stream_state),
        )
        .route(route::LIVE_LICENSE, get(handlers::live_license))
}
