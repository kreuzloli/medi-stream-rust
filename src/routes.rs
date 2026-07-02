use crate::account::handlers as account_handlers;
use crate::auth::handlers as auth_handlers;
use crate::hospital::handlers as hospital_handlers;
use crate::state::AppState;
use axum::routing::{delete, get, post};
use axum::Router;
use tower_http::cors::CorsLayer;

pub fn router(state: AppState) -> Router {
    // Router 是 Axum 的路由表。每个 route 把 HTTP method 绑定到 handlers.rs 里的函数。
    // with_state 会把 AppState 注入到所有 handler，类似 Spring 里的依赖注入。
    Router::new()
        .route("/auth/login", post(auth_handlers::login))
        .route("/auth/logout", get(auth_handlers::logout))
        .route("/auth/me", get(auth_handlers::me))
        .route("/catalog/departments", get(hospital_handlers::departments))
        .route(
            "/catalog/departments/:dept_id/diseases",
            get(hospital_handlers::diseases_by_department),
        )
        .route("/catalog/full", get(hospital_handlers::full_catalog))
        .route(
            "/hospitals",
            get(hospital_handlers::page_hospitals).post(hospital_handlers::create_hospital),
        )
        .route(
            "/hospitals/:id",
            get(hospital_handlers::get_hospital)
                .put(hospital_handlers::update_hospital)
                .delete(hospital_handlers::delete_hospital),
        )
        .route("/auth/register", post(auth_handlers::register))
        .route("/account", get(account_handlers::get_account))
        .route("/account/bind/login", post(account_handlers::bind_account))
        .route(
            "/account/unbind/:login_id",
            delete(account_handlers::unbind_account),
        )
        // 当前先放开 CORS，方便前端本地调试；上线时可以改成白名单域名。
        .layer(CorsLayer::permissive())
        .with_state(state)
}
