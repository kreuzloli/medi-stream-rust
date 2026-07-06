use crate::error::AppError;
use crate::tencent_cloud::tencent_live_model::{LiveUrlConfig, LiveUrlsResp};

#[derive(Debug, Clone, Copy)]
pub enum LiveProtocol {
    Webrtc,
    Rtmp,
    HttpFlv,
    Hls,
}

impl LiveProtocol {
    /// 返回当前播放协议对应的 URL 前缀。
    fn prefix(self) -> &'static str {
        match self {
            LiveProtocol::Webrtc => "webrtc://",
            LiveProtocol::Rtmp => "rtmp://",
            LiveProtocol::HttpFlv | LiveProtocol::Hls => "https://",
        }
    }

    /// 返回当前播放协议对应的文件后缀。
    fn suffix(self) -> &'static str {
        match self {
            LiveProtocol::Webrtc | LiveProtocol::Rtmp => "",
            LiveProtocol::HttpFlv => ".flv",
            LiveProtocol::Hls => ".m3u8",
        }
    }
}

/// 按腾讯云规则生成推流 URL、播放 URL 和可选转码播放 URL。
pub fn build_live_urls(
    config: &LiveUrlConfig,
    stream_name: &str,
    ttl_seconds: Option<i64>,
    transcode_template: Option<&str>,
    now_epoch_seconds: i64,
) -> Result<LiveUrlsResp, AppError> {
    validate_config(config)?;
    let stream_name = require_not_blank("streamName", stream_name)?;
    let ttl = ttl_seconds
        .filter(|ttl| *ttl > 0)
        .unwrap_or(config.default_ttl_seconds);
    let expire_at = now_epoch_seconds + ttl;
    let tx_time_hex = to_upper_hex(expire_at);
    let transcode_template = transcode_template
        .map(str::trim)
        .filter(|template| !template.is_empty());

    let push_rtmp = build_push_url(
        LiveProtocol::Rtmp,
        &config.push_domain,
        &config.app_name,
        stream_name,
        &config.push_key,
        &tx_time_hex,
    );
    let push_webrtc = build_push_url(
        LiveProtocol::Webrtc,
        &config.push_domain,
        &config.app_name,
        stream_name,
        &config.push_key,
        &tx_time_hex,
    );
    let play_webrtc = build_play_url(
        LiveProtocol::Webrtc,
        &config.play_domain,
        &config.app_name,
        stream_name,
        None,
        &config.play_key,
        &tx_time_hex,
    );
    let play_rtmp = build_play_url(
        LiveProtocol::Rtmp,
        &config.play_domain,
        &config.app_name,
        stream_name,
        None,
        &config.play_key,
        &tx_time_hex,
    );
    let play_flv = build_play_url(
        LiveProtocol::HttpFlv,
        &config.play_domain,
        &config.app_name,
        stream_name,
        None,
        &config.play_key,
        &tx_time_hex,
    );
    let play_hls = build_play_url(
        LiveProtocol::Hls,
        &config.play_domain,
        &config.app_name,
        stream_name,
        None,
        &config.play_key,
        &tx_time_hex,
    );
    let play_flv_transcoded = transcode_template.map(|template| {
        build_play_url(
            LiveProtocol::HttpFlv,
            &config.play_domain,
            &config.app_name,
            stream_name,
            Some(template),
            &config.play_key,
            &tx_time_hex,
        )
    });
    let play_hls_transcoded = transcode_template.map(|template| {
        build_play_url(
            LiveProtocol::Hls,
            &config.play_domain,
            &config.app_name,
            stream_name,
            Some(template),
            &config.play_key,
            &tx_time_hex,
        )
    });

    Ok(LiveUrlsResp {
        stream_name: stream_name.to_string(),
        expire_at_epoch_seconds: expire_at,
        tx_time_hex,
        push_webrtc,
        push_rtmp,
        play_webrtc,
        play_rtmp,
        play_flv,
        play_hls,
        transcode_template: transcode_template.map(str::to_string),
        play_flv_transcoded,
        play_hls_transcoded,
    })
}

/// 生成带 txSecret 和 txTime 的 RTMP 推流 URL。
pub fn build_push_url(
    protocol: LiveProtocol,
    push_domain: &str,
    app_name: &str,
    stream_name: &str,
    push_key: &str,
    tx_time_hex: &str,
) -> String {
    // 腾讯云推流防盗链签名要求按 pushKey + streamName + txTime 拼 MD5。
    let base = format!(
        "{}://{}/{}/{}",
        protocol.prefix(),
        trim_slash(push_domain),
        trim_slash(app_name),
        stream_name
    );
    let tx_secret = md5_hex(&format!("{push_key}{stream_name}{tx_time_hex}"));

    format!("{base}?txSecret={tx_secret}&txTime={tx_time_hex}")
}

/// 生成指定协议的播放 URL，支持源流和转码流。
pub fn build_play_url(
    protocol: LiveProtocol,
    play_domain: &str,
    app_name: &str,
    stream_name: &str,
    transcode_template: Option<&str>,
    play_key: &str,
    tx_time_hex: &str,
) -> String {
    // 播放转码流的签名对象是 streamName_template，必须和最终 URL 路径保持一致。
    let stream_part = transcode_template
        .map(str::trim)
        .filter(|template| !template.is_empty())
        .map(|template| format!("{stream_name}_{template}"))
        .unwrap_or_else(|| stream_name.to_string());
    let base = format!(
        "{}{}/{}/{}{}",
        protocol.prefix(),
        trim_slash(play_domain),
        trim_slash(app_name),
        stream_part,
        protocol.suffix()
    );
    let tx_secret = md5_hex(&format!("{play_key}{stream_part}{tx_time_hex}"));

    format!("{base}?txSecret={tx_secret}&txTime={tx_time_hex}")
}

/// 把过期 Unix 秒转换成腾讯云要求的大写十六进制 txTime。
pub fn to_upper_hex(unix_seconds: i64) -> String {
    format!("{unix_seconds:X}")
}

/// 校验 URL 生成所需的腾讯云直播配置。
fn validate_config(config: &LiveUrlConfig) -> Result<(), AppError> {
    require_not_blank("TENCENT_LIVE_APP_NAME", &config.app_name)?;
    require_not_blank("TENCENT_LIVE_PUSH_DOMAIN", &config.push_domain)?;
    require_not_blank("TENCENT_LIVE_PLAY_DOMAIN", &config.play_domain)?;
    require_not_blank("TENCENT_LIVE_PUSH_KEY", &config.push_key)?;
    require_not_blank("TENCENT_LIVE_PLAY_KEY", &config.play_key)?;
    if config.default_ttl_seconds <= 0 {
        return Err(AppError::BadRequest(
            "TENCENT_LIVE_DEFAULT_TTL_SECONDS必须大于0".to_string(),
        ));
    }

    Ok(())
}

/// 读取必填字段，缺失或空值时返回业务错误。
fn require_not_blank<'a>(name: &str, value: &'a str) -> Result<&'a str, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(format!("{name}不能为空")));
    }

    Ok(trimmed)
}

/// 计算小写 MD5 十六进制字符串。
fn md5_hex(value: &str) -> String {
    format!("{:x}", md5::compute(value.as_bytes()))
}

/// 去掉域名或 appName 首尾空白和斜杠。
fn trim_slash(value: &str) -> &str {
    value.trim().trim_matches('/')
}
