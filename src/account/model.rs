use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    // Option<T> 表示数据库或请求 JSON 里这个字段可以为空。
    // serde 的 camelCase 保持接口字段名和 Java DTO 一致。
    pub id: Option<u64>,
    pub user_code: Option<String>,
    pub nickname: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: Option<i32>,
    pub version: Option<i32>,
    pub is_deleted: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountPageQuery {
    pub page: Option<u64>,
    pub size: Option<u64>,
    pub user_code: Option<String>,
}
