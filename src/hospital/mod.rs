pub mod catalog_model;
pub mod catalog_repository;
pub mod catalog_service;
pub mod handlers;
pub mod hospital_model;
pub mod hospital_repository;
pub mod hospital_service;

use crate::common::constants::route;
use crate::state::AppState;
use axum::Router;
use axum::routing::get;

/// 注册医院及科室疾病目录接口。
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(route::CATALOG_DEPARTMENTS, get(handlers::departments))
        .route(
            route::CATALOG_DEPARTMENT_DISEASES,
            get(handlers::diseases_by_department),
        )
        .route(route::CATALOG_FULL, get(handlers::full_catalog))
        .route(
            route::HOSPITALS,
            get(handlers::page_hospitals).post(handlers::create_hospital),
        )
        .route(
            route::HOSPITAL_BY_ID,
            get(handlers::get_hospital)
                .put(handlers::update_hospital)
                .delete(handlers::delete_hospital),
        )
}
