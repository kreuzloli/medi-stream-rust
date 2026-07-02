use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Hospital {
    pub id: u64,
    pub hospital_name: String,
    pub hospital_code: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    pub address: Option<String>,
    pub sort_no: i32,
    pub status: i8,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveHospitalReq {
    pub hospital_name: String,
    pub hospital_code: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    pub address: Option<String>,
    pub sort_no: Option<i32>,
    pub status: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HospitalPageQuery {
    pub page: Option<u64>,
    pub size: Option<u64>,
    pub hospital_name: Option<String>,
    pub hospital_code: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    pub status: Option<i32>,
}
