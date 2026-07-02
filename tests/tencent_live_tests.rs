use medi_stream_rust::tencent_cloud::tencent_live_model::DescribeLiveStreamStateReq;
use medi_stream_rust::tencent_cloud::tencent_live_model::LiveUrlConfig;
use medi_stream_rust::tencent_cloud::tencent_live_signer::{
    build_live_authorization, LiveCredential,
};
use medi_stream_rust::tencent_cloud::tencent_live_url_generator::{
    build_live_urls, build_play_url, build_push_url, PlayProtocol,
};

/// 验证直播业务模型的核心行为。
#[test]
fn live_stream_state_request_rejects_blank_fields() {
    let req = DescribeLiveStreamStateReq {
        app_name: "live".to_string(),
        domain_name: "push.example.com".to_string(),
        stream_name: "  ".to_string(),
    };

    let err = req
        .validate()
        .expect_err("blank stream name must be rejected");

    assert!(err.to_string().contains("streamName不能为空"));
}

/// 验证直播业务模型的核心行为。
#[test]
fn live_authorization_uses_tencent_tc3_format() {
    let req = DescribeLiveStreamStateReq {
        app_name: "live".to_string(),
        domain_name: "push.example.com".to_string(),
        stream_name: "stream001".to_string(),
    };
    let credential = LiveCredential {
        secret_id: "AKIDEXAMPLE".to_string(),
        secret_key: "SECRETEXAMPLE".to_string(),
    };

    let authorization =
        build_live_authorization(&credential, 1_704_067_200, &req).expect("signature should build");

    assert!(authorization
        .starts_with("TC3-HMAC-SHA256 Credential=AKIDEXAMPLE/2024-01-01/live/tc3_request"));
    assert!(authorization.contains("SignedHeaders=content-type;host"));
    assert!(authorization.contains("Signature="));
}

/// 验证直播业务模型的核心行为。
#[test]
fn live_url_generator_builds_signed_push_and_play_urls() {
    let config = LiveUrlConfig {
        app_name: "medi-stream".to_string(),
        push_domain: "push.genwhole.com".to_string(),
        play_domain: "live.genwhole.com".to_string(),
        push_key: "push-secret".to_string(),
        play_key: "play-secret".to_string(),
        default_ttl_seconds: 86_400,
    };

    let urls = build_live_urls(&config, "stream001", Some(60), Some("hd"), 1_700_000_000)
        .expect("live urls should build");

    assert_eq!(urls.stream_name, "stream001");
    assert_eq!(urls.expire_at_epoch_seconds, 1_700_000_060);
    assert_eq!(urls.tx_time_hex, "6553F13C");
    assert_eq!(
        urls.push_rtmp,
        build_push_url(
            "push.genwhole.com",
            "medi-stream",
            "stream001",
            "push-secret",
            "6553F13C",
        )
    );
    assert_eq!(
        urls.play_webrtc,
        build_play_url(
            PlayProtocol::Webrtc,
            "live.genwhole.com",
            "medi-stream",
            "stream001",
            None,
            "play-secret",
            "6553F13C",
        )
    );
    let expected_flv_transcoded = build_play_url(
        PlayProtocol::HttpFlv,
        "live.genwhole.com",
        "medi-stream",
        "stream001",
        Some("hd"),
        "play-secret",
        "6553F13C",
    );
    let expected_hls_transcoded = build_play_url(
        PlayProtocol::Hls,
        "live.genwhole.com",
        "medi-stream",
        "stream001",
        Some("hd"),
        "play-secret",
        "6553F13C",
    );
    assert_eq!(
        urls.play_flv_transcoded.as_deref(),
        Some(expected_flv_transcoded.as_str())
    );
    assert_eq!(
        urls.play_hls_transcoded.as_deref(),
        Some(expected_hls_transcoded.as_str())
    );
}

/// 验证直播业务模型的核心行为。
#[test]
fn live_url_generator_rejects_blank_stream_name() {
    let config = LiveUrlConfig {
        app_name: "medi-stream".to_string(),
        push_domain: "push.genwhole.com".to_string(),
        play_domain: "live.genwhole.com".to_string(),
        push_key: "push-secret".to_string(),
        play_key: "play-secret".to_string(),
        default_ttl_seconds: 86_400,
    };

    let err = build_live_urls(&config, "  ", None, None, 1_700_000_000)
        .expect_err("blank stream name must be rejected");

    assert!(err.to_string().contains("streamName不能为空"));
}
