pub mod handlers;
pub mod live_model;
pub mod live_repository;
pub mod live_service;

use crate::common::constants::route;
use crate::state::AppState;
use axum::routing::get;
use axum::Router;

/// 注册登录用户访问的直播观看接口。
pub fn routes() -> Router<AppState> {
    Router::new().route(
        route::LIVE_WATCH_BY_ROOM_CODE,
        get(handlers::watch_live_room),
    )
}
