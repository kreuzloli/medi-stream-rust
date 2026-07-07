use crate::account::handlers as account_handlers;
use crate::auth::handlers as auth_handlers;
use crate::common::constants::route;
use crate::hospital::handlers as hospital_handlers;
use crate::state::AppState;
use crate::tencent_cloud::handlers as tencent_cloud_handlers;
use crate::wechat::handlers as wechat_handlers;
use axum::routing::{delete, get, post};
use axum::Router;
use tower_http::cors::CorsLayer;

/// 组装全局路由表，并把共享 AppState 注入到每个 handler。
pub fn router(state: AppState) -> Router {
    Router::new()
        .route(route::AUTH_LOGIN, post(auth_handlers::login))
        .route(route::AUTH_LOGOUT, get(auth_handlers::logout))
        .route(route::AUTH_ME, get(auth_handlers::me))
        .route(
            route::CATALOG_DEPARTMENTS,
            get(hospital_handlers::departments),
        )
        .route(
            route::CATALOG_DEPARTMENT_DISEASES,
            get(hospital_handlers::diseases_by_department),
        )
        .route(route::CATALOG_FULL, get(hospital_handlers::full_catalog))
        .route(
            route::HOSPITALS,
            get(hospital_handlers::page_hospitals).post(hospital_handlers::create_hospital),
        )
        .route(
            route::HOSPITAL_BY_ID,
            get(hospital_handlers::get_hospital)
                .put(hospital_handlers::update_hospital)
                .delete(hospital_handlers::delete_hospital),
        )
        .route(route::AUTH_REGISTER, post(auth_handlers::register))
        .route(
            route::LIVE_URLS,
            get(tencent_cloud_handlers::generate_live_urls),
        )
        .route(
            route::LIVE_STREAM_STATE,
            post(tencent_cloud_handlers::describe_live_stream_state),
        )
        .route(route::ACCOUNT, get(account_handlers::get_account))
        .route(
            route::ACCOUNT_BIND_LOGIN,
            post(account_handlers::bind_account),
        )
        .route(
            route::ACCOUNT_UNBIND,
            delete(account_handlers::unbind_account),
        )
        .route(
            route::WECHAT_CALLBACK,
            get(wechat_handlers::check_signature),
        )
                .route(
            route::WECHAT_RELOAD_ACCESS_TOKEN,
            get(wechat_handlers::reload_access_token),
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}
