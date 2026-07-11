use axum::http::header::CONTENT_TYPE;
use axum::http::HeaderMap;
use axum::routing::get;
use axum::Router;
use medi_stream_rust::common::HttpClient;
use medi_stream_rust::tencent_cloud::tencent_live_license::{
    fetch_live_license, LiveLicenseConfig,
};
use tokio::net::TcpListener;

#[tokio::test]
async fn fetch_live_license_returns_raw_body_without_sending_license_key() {
    let upstream = Router::new().route(
        "/license",
        get(|headers: HeaderMap| async move {
            assert!(!headers.values().any(|value| value == "private-license-key"));
            (
                [(CONTENT_TYPE, "application/octet-stream")],
                "license-content",
            )
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test listener should bind");
    let address = listener.local_addr().expect("listener should have address");
    tokio::spawn(async move {
        axum::serve(listener, upstream)
            .await
            .expect("test upstream should run");
    });

    let config = LiveLicenseConfig {
        url: format!("http://{address}/license"),
        key: "private-license-key".to_string(),
    };
    let http = HttpClient::new(5).expect("http client should build");

    let response = fetch_live_license(&http, &config)
        .await
        .expect("license proxy should succeed");

    assert_eq!(response.content_type, "application/octet-stream");
    assert_eq!(response.body.as_slice(), b"license-content");
}
