pub mod handlers;
pub mod wechat_cache;
pub mod wechat_enum;
pub mod wechat_model;
pub mod wechat_service;

use crate::common::constants::route::{
    self, AUTH_WECHAT_QRCODE, AUTH_WECHAT_QRCODE_CALLBACK, AUTH_WECHAT_QRCODE_FILE,
    AUTH_WECHAT_QRCODE_REGISTER, AUTH_WECHAT_STATUS,
};
use crate::state::AppState;
use axum::routing::{get, post};
use axum::{extract::DefaultBodyLimit, Router};

/// 注册微信服务器回调、OAuth 和扫码登录接口。
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
        .route(AUTH_WECHAT_QRCODE_CALLBACK, get(handlers::qrcode_callback))
        .route(AUTH_WECHAT_QRCODE_REGISTER, post(handlers::qrcode_register))
        .route(
            AUTH_WECHAT_QRCODE_FILE,
            post(handlers::qrcode_upload_file).layer(DefaultBodyLimit::disable()),
        )
        .route(AUTH_WECHAT_STATUS, get(handlers::get_status))
}
