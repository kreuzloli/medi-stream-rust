use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct FileObject {
    pub id: u64,
    pub file_name: String,
    pub file_url: String,
    pub mime_type: Option<String>,
    pub file_size: Option<u64>,
    pub sha256: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveFileObjectReq {
    pub file_name: String,
    pub file_url: String,
    pub mime_type: Option<String>,
    pub file_size: Option<u64>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileObjectPageQuery {
    pub page: Option<u64>,
    pub size: Option<u64>,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub sha256: Option<String>,
}
