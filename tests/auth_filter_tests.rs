use axum::body::{to_bytes, Body};
use axum::http::{header, Method, Request, StatusCode};
use axum::routing::get;
use axum::{Extension, Router};
use medi_stream_rust::common::constants::cache::ACCOUNT_CACHE_SECONDS;
use medi_stream_rust::common::jwt::{Claims, CurrentToken, CurrentUser};
use medi_stream_rust::common::{HttpClient, JwtKeys};
use medi_stream_rust::config::{FileStorageConfig, Settings};
use medi_stream_rust::routes::router;
use medi_stream_rust::state::AppState;
use sqlx::mysql::MySqlPoolOptions;
use tower::ServiceExt;

fn test_state() -> AppState {
    let settings = Settings {
        server_addr: "127.0.0.1:0".to_string(),
        database_url: "mysql://root:password@127.0.0.1:3306/medi".to_string(),
        redis_url: "redis://127.0.0.1:6379/0".to_string(),
        jwt_secret_base64: "dGVzdC1zZWNyZXQtdGVzdC1zZWNyZXQ=".to_string(),
        jwt_issuer: "medi-stream-test".to_string(),
        jwt_ttl_seconds: 3600,
        mysql_max_connections: 1,
        http_timeout_seconds: 1,
        file_storage: FileStorageConfig {
            root_dir: std::path::PathBuf::from("/tmp/medi-stream-test-uploads"),
            public_prefix: "/uploads".to_string(),
            max_size_bytes: 100 * 1024 * 1024,
        },
        tencent_live_credential: None,
        tencent_live_url_config: None,
        tencent_live_license_config: None,
        wechat_token: None,
        wechat_app_id: None,
        wechat_app_secret: None,
        wechat_encoding_aes_key: None,
        wechat_access_token_expire_seconds: None,
        web_base_url: "http://127.0.0.1:5173".to_string(),
        wechat_oauth_callback_base_url: None,
    };

    AppState {
        db: MySqlPoolOptions::new()
            .connect_lazy(&settings.database_url)
            .expect("test database URL should be valid"),
        redis: None,
        jwt: JwtKeys::from_settings(&settings).expect("test JWT settings should be valid"),
        http: HttpClient::new(settings.http_timeout_seconds)
            .expect("test HTTP client should build"),
        file_storage: settings.file_storage.clone(),
        tencent_live_credential: None,
        tencent_live_url_config: None,
        tencent_live_license_config: None,
        wechat_token: None,
        wechat_app_id: None,
        wechat_app_secret: None,
        wechat_encoding_aes_key: None,
        wechat_access_token_expire_seconds: None,
        web_base_url: settings.web_base_url,
        wechat_oauth_callback_base_url: None,
    }
}

#[tokio::test]
async fn token_cache_ttl_matches_jwt_configuration_instead_of_account_cache_ttl() {
    let state = test_state();

    assert_eq!(state.jwt.token_ttl_seconds(), 3600);
    assert_ne!(state.jwt.token_ttl_seconds(), ACCOUNT_CACHE_SECONDS);
}

async fn authenticated_context(
    CurrentUser(claims): CurrentUser,
    CurrentToken(token): CurrentToken,
) -> String {
    format!("{}:{token}", claims.uid.unwrap_or_default())
}

#[tokio::test]
async fn authentication_extractors_read_only_the_context_inserted_by_middleware() {
    let claims = Claims {
        iss: "medi-stream-test".to_string(),
        sub: "doctor@example.com".to_string(),
        iat: 1,
        exp: i64::MAX,
        roles: vec!["USER".to_string()],
        uid: Some(100),
    };
    let authenticated = Router::new()
        .route("/", get(authenticated_context))
        .layer(Extension(CurrentToken("current-token".to_string())))
        .layer(Extension(CurrentUser(claims)));

    let response = authenticated
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&body[..], b"100:current-token");

    let missing_context = Router::new()
        .route("/", get(authenticated_context))
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(missing_context.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_route_rejects_valid_jwt_when_token_cache_is_unavailable() {
    let state = test_state();
    let token = state
        .jwt
        .generate_token("doctor@example.com", vec!["USER".to_string()], Some(100))
        .expect("test token should be generated");

    let response = router(state)
        .oneshot(
            Request::builder()
                .uri("/auth/me")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn production_router_keeps_public_and_protected_routes_separate() {
    let app = router(test_state());

    let login = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/auth/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

    let register = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/auth/register")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(register.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

    let missing_me_token = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing_me_token.status(), StatusCode::UNAUTHORIZED);

    let missing_account_token = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/account")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing_account_token.status(), StatusCode::UNAUTHORIZED);

    let missing_bind_token = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/account/bind/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing_bind_token.status(), StatusCode::UNAUTHORIZED);

    let missing_unbind_token = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/account/unbind/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing_unbind_token.status(), StatusCode::UNAUTHORIZED);

    let missing_live_watch_token = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/live/watch/ROOM001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing_live_watch_token.status(), StatusCode::UNAUTHORIZED);

    let missing_file_upload_token = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/files/upload")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing_file_upload_token.status(), StatusCode::UNAUTHORIZED);

    let public_wechat_callback = app
        .oneshot(
            Request::builder()
                .uri("/wechat/callback")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_ne!(public_wechat_callback.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_route_rejects_invalid_bearer_token() {
    let response = router(test_state())
        .oneshot(
            Request::builder()
                .uri("/auth/me")
                .header(header::AUTHORIZATION, "Token invalid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
