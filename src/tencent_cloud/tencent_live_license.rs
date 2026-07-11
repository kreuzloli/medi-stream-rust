use crate::common::HttpClient;
use crate::error::AppError;
use reqwest::header::CONTENT_TYPE;

#[derive(Debug, Clone)]
pub struct LiveLicenseConfig {
    pub url: String,
    pub key: String,
}

pub struct LiveLicenseResponse {
    pub body: Vec<u8>,
    pub content_type: String,
}

/// 由服务端下载播放器 License，避免真实 URL 和 Key 进入前端构建产物。
pub async fn fetch_live_license(
    http: &HttpClient,
    config: &LiveLicenseConfig,
) -> Result<LiveLicenseResponse, AppError> {
    let response = http.raw().get(&config.url).send().await?;
    let status = response.status();

    if !status.is_success() {
        tracing::warn!(
            status = status.as_u16(),
            "tencent live license upstream returned non-success status"
        );
        return Err(AppError::ExternalApi {
            service: "tencent_live_license".to_string(),
            status: status.as_u16(),
            body: "license request failed".to_string(),
        });
    }

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();
    let body = response.bytes().await?.to_vec();

    Ok(LiveLicenseResponse { body, content_type })
}
