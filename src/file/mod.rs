pub mod file_model;
pub mod file_repository;
pub mod file_service;
pub mod file_storage;
pub mod handlers;
use crate::common::constants::route;
use crate::state::AppState;
use axum::routing::post;
use axum::{extract::DefaultBodyLimit, Router};

/// 注册登录用户使用的通用文件上传接口。
///
/// Axum 默认请求体限制不适用于大文件；实际大小由流式存储逻辑按 FILE_MAX_SIZE_BYTES 校验。
/// Admin 端的全量分页和任意文件详情接口不在主站暴露，避免普通用户枚举证件文件。
pub fn routes() -> Router<AppState> {
    Router::new().route(
        route::FILE_UPLOAD,
        post(handlers::upload_file).layer(DefaultBodyLimit::disable()),
    )
}
