pub mod handlers;

use crate::common::constants::route;
use crate::state::AppState;
use axum::Router;
use axum::routing::{get, post};

/// 注册登录、退出、当前用户和注册接口。
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(route::AUTH_LOGIN, post(handlers::login))
        .route(route::AUTH_LOGOUT, get(handlers::logout))
        .route(route::AUTH_ME, get(handlers::me))
        .route(route::AUTH_REGISTER, post(handlers::register))
}
