pub mod handlers;
pub mod wechat_cache;
pub mod wechat_enum;
pub mod wechat_model;
pub mod wechat_service;

use crate::common::constants::route::{self, AUTH_WECHAT_QRCODE, AUTH_WECHAT_STATUS};
use crate::state::AppState;
use axum::routing::get;
use axum::Router;

/// 注册微信回调、Access Token 刷新和 OAuth 接口。
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(route::WECHAT_CALLBACK, get(handlers::check_signature))
        .route(
            route::WECHAT_RELOAD_ACCESS_TOKEN,
            get(handlers::reload_access_token),
        )
        .route(
            route::WECHAT_OAUTH_AUTHORIZE,
            get(handlers::oauth_authorize),
        )
        .route(route::WECHAT_OAUTH_CALLBACK, get(handlers::oauth_callback))
        .route(AUTH_WECHAT_QRCODE, get(handlers::create_qrcode))
        .route(AUTH_WECHAT_STATUS, get(handlers::get_status))
}
