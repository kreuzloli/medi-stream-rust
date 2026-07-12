use crate::state::AppState;
use axum::Router;
use tower_http::cors::CorsLayer;

/// 组装全局路由表，并把共享 AppState 注入到每个领域路由。
pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(crate::auth::routes())
        .merge(crate::account::routes())
        .merge(crate::hospital::routes())
        .merge(crate::tencent_cloud::routes())
        .merge(crate::wechat::routes())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
