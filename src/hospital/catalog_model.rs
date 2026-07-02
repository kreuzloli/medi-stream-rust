use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Department {
    // FromRow 让 SQLx 可以把 SELECT 结果直接映射成结构体。
    // 字段名要和 SQL 查询返回列名一致，例如 dept_name、sort_no。
    pub id: u64,
    pub dept_name: String,
    pub dept_code: Option<String>,
    pub sort_no: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct Disease {
    pub id: u64,
    pub dept_id: u64,
    pub disease_name: String,
    pub disease_code: Option<String>,
    pub keywords: Option<String>,
    pub sort_no: i32,
    pub status: i8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiseaseDto {
    pub id: u64,
    pub disease_name: String,
    pub disease_code: Option<String>,
    pub keywords: Option<String>,
    pub sort: i32,
    pub status: i32,
}

impl From<Disease> for DiseaseDto {
    fn from(value: Disease) -> Self {
        // From trait 是 Rust 常用的类型转换写法，这里相当于 Java 的 entity -> DTO。
        Self {
            id: value.id,
            disease_name: value.disease_name,
            disease_code: value.disease_code,
            keywords: value.keywords,
            sort: value.sort_no,
            status: i32::from(value.status),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DepartmentWithDiseasesDto {
    pub dept_id: u64,
    pub dept_name: String,
    pub dept_code: Option<String>,
    pub diseases_preview: Option<String>,
    pub sort: i32,
    pub diseases: Vec<DiseaseDto>,
}

impl DepartmentWithDiseasesDto {
    pub fn from_department(value: Department, diseases: Vec<DiseaseDto>) -> Self {
        // Vec<T> 是 Rust 的动态数组，对应 Java 里的 List<T>。
        Self {
            dept_id: value.id,
            dept_name: value.dept_name,
            dept_code: value.dept_code,
            diseases_preview: None,
            sort: value.sort_no,
            diseases,
        }
    }

    pub fn join_disease_names_ellipsis(&mut self) {
        self.join_disease_names_ellipsis_with_max(12);
    }

    pub fn join_disease_names_ellipsis_with_max(&mut self, max_len: usize) {
        // iter() 是借用遍历，不会消耗 diseases；collect::<Vec<_>>() 再 join 成字符串。
        let joined = self
            .diseases
            .iter()
            .map(|disease| disease.disease_name.trim())
            .filter(|name| !name.is_empty())
            .collect::<Vec<_>>()
            .join(" · ");

        if joined.is_empty() {
            return;
        }

        let preview = if joined.chars().count() <= max_len {
            joined
        } else {
            // 按 char 截断，避免中文 UTF-8 字节被截坏。
            format!("{}...", joined.chars().take(max_len).collect::<String>())
        };
        self.diseases_preview = Some(preview);
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepartmentQuery {
    pub include_diseases: Option<bool>,
}
