use crate::common::jwt::authenticate_user;
use crate::state::AppState;
use axum::{middleware, Router};
use tower_http::cors::CorsLayer;

/// 组装全局路由表，并把共享 AppState 注入到每个领域路由。
pub fn router(state: AppState) -> Router {
    let protected_routes = Router::new()
        .merge(crate::auth::routes())
        .merge(crate::account::routes())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            authenticate_user,
        ));

    Router::new()
        .merge(crate::auth::public_routes())
        .merge(protected_routes)
        .merge(crate::hospital::routes())
        .merge(crate::tencent_cloud::routes())
        .merge(crate::wechat::routes())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
