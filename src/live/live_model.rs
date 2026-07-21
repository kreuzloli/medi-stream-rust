use crate::tencent_cloud::tencent_live_model::LiveUrlsResp;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LiveRoom {
    pub id: u64,
    // 普通用户和管理员使用独立所有者字段，数据库和 Service 都保证二选一。
    pub owner_user_id: Option<u64>,
    pub owner_admin_id: Option<u64>,
    pub room_code: String,
    pub title: String,
    pub description: Option<String>,
    pub cover_file_id: Option<u64>,
    pub department_id: Option<u64>,
    pub disease_id: Option<u64>,
    pub is_top: i8,
    pub status: i8,
    pub is_deleted: i8,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub start_time: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveRoomDetail {
    // 一个直播房间可以绑定多路流；详情响应把房间主表和流列表一起返回，避免调用方再拼装。
    #[serde(flatten)]
    pub room: LiveRoom,
    pub streams: Vec<LiveRoomStream>,
}

/// 登录用户进入直播间时所需的完整播放信息。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveWatchResp {
    pub room: LiveRoom,
    pub stream: LiveRoomStream,
    pub urls: LiveUrlsResp,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLiveRoomReq {
    pub owner_user_id: Option<u64>,
    pub owner_admin_id: Option<u64>,
    pub room_code: String,
    pub title: String,
    pub description: Option<String>,
    pub cover_file_id: Option<u64>,
    pub department_id: Option<u64>,
    pub disease_id: Option<u64>,
    pub is_top: Option<i32>,
    pub status: Option<i32>,
    pub start_time: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveRoomPageQuery {
    pub page: Option<u64>,
    pub size: Option<u64>,
    pub owner_user_id: Option<u64>,
    pub owner_admin_id: Option<u64>,
    pub department_id: Option<u64>,
    pub disease_id: Option<u64>,
    pub is_top: Option<i32>,
    pub room_code: Option<String>,
    pub title: Option<String>,
    pub status: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LiveRoomStream {
    pub id: u64,
    pub room_id: u64,
    pub stream_code: String,
    pub stream_name: String,
    pub title: Option<String>,
    pub sort_no: i32,
    pub is_default: i8,
    pub status: i8,
    pub is_deleted: i8,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLiveRoomStreamReq {
    // room_id 是直播流归属边界；stream_code/stream_name 只在同一房间内唯一。
    pub room_id: u64,
    pub stream_code: String,
    pub stream_name: String,
    pub title: Option<String>,
    pub sort_no: Option<i32>,
    pub is_default: Option<i32>,
    pub status: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveRoomStreamPageQuery {
    pub page: Option<u64>,
    pub size: Option<u64>,
    pub room_id: Option<u64>,
    pub stream_code: Option<String>,
    pub stream_name: Option<String>,
    pub status: Option<i32>,
}
