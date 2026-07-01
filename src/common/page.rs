use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub records: Vec<T>,
    pub total: u64,
    pub size: u64,
    pub current: u64,
    pub pages: u64,
}
