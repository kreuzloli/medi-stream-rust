pub mod handlers;
pub mod wechat_model;
pub mod wechat_service;

use crate::common::constants::route;
use crate::state::AppState;
use axum::Router;
use axum::routing::get;

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
        .route(
            route::WECHAT_OAUTH_CALLBACK,
            get(handlers::oauth_callback),
        )
}
