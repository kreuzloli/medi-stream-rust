pub mod account_model;
pub mod account_repository;
pub mod account_service;
pub mod handlers;

use crate::common::constants::route;
use crate::state::AppState;
use axum::Router;
use axum::routing::{delete, get, post};

/// 注册账号详情、绑定和解绑接口。
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(route::ACCOUNT, get(handlers::get_account))
        .route(route::ACCOUNT_BIND_LOGIN, post(handlers::bind_account))
        .route(route::ACCOUNT_UNBIND, delete(handlers::unbind_account))
}
