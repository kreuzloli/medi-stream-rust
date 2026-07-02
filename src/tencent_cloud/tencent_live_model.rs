use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DescribeLiveStreamStateReq {
    pub app_name: String,
    pub domain_name: String,
    pub stream_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DescribeLiveStreamStateResp {
    pub response: serde_json::Value,
}
