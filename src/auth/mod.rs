pub mod handlers;

use crate::common::constants::route;
use crate::state::AppState;
use axum::routing::{get, post};
use axum::Router;

/// 注册无需用户认证的登录和注册接口。
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route(route::AUTH_LOGIN, post(handlers::login))
        .route(route::AUTH_REGISTER, post(handlers::register))
}

/// 注册需要用户认证的退出和当前用户接口。
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(route::AUTH_LOGOUT, get(handlers::logout))
        .route(route::AUTH_ME, get(handlers::me))
}
