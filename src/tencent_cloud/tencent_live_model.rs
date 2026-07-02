use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct LiveUrlConfig {
    pub app_name: String,
    pub push_domain: String,
    pub play_domain: String,
    pub push_key: String,
    pub play_key: String,
    pub default_ttl_seconds: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveUrlsQuery {
    pub stream_name: String,
    pub ttl_seconds: Option<i64>,
    pub transcode_template: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LiveUrlsResp {
    pub stream_name: String,
    pub expire_at_epoch_seconds: i64,
    pub tx_time_hex: String,
    pub push_rtmp: String,
    pub play_webrtc: String,
    pub play_rtmp: String,
    pub play_flv: String,
    pub play_hls: String,
    pub transcode_template: Option<String>,
    pub play_flv_transcoded: Option<String>,
    pub play_hls_transcoded: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DescribeLiveStreamStateReq {
    pub app_name: String,
    pub domain_name: String,
    pub stream_name: String,
}

impl DescribeLiveStreamStateReq {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.app_name.trim().is_empty() {
            return Err(AppError::BadRequest("appName不能为空".to_string()));
        }
        if self.domain_name.trim().is_empty() {
            return Err(AppError::BadRequest("domainName不能为空".to_string()));
        }
        if self.stream_name.trim().is_empty() {
            return Err(AppError::BadRequest("streamName不能为空".to_string()));
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DescribeLiveStreamStateResp {
    pub response: serde_json::Value,
}
