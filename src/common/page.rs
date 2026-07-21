use crate::common::constants::page::{DEFAULT_PAGE, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
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

/// 规范分页参数并计算数据库偏移量。
pub fn page_params(page: Option<u64>, size: Option<u64>) -> (u64, u64, u64) {
    let page = page.unwrap_or(DEFAULT_PAGE).max(DEFAULT_PAGE);
    let size = size.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);
    (page, size, (page - 1) * size)
}

/// 将查询记录和总数转换为统一分页响应。
pub fn to_page<T>(records: Vec<T>, total: i64, page: u64, size: u64) -> Page<T> {
    let total = total.max(0) as u64;
    Page {
        records,
        total,
        size,
        current: page,
        pages: if total == 0 { 0 } else { total.div_ceil(size) },
    }
}
