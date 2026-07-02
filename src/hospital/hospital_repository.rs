use crate::common::Page;
use crate::error::AppError;
use crate::hospital::hospital_model::{Hospital, HospitalPageQuery, SaveHospitalReq};
use sqlx::{MySql, MySqlPool, QueryBuilder, Row};

/// 向数据库插入记录，并返回新记录 ID。
pub async fn insert_hospital(db: &MySqlPool, req: &SaveHospitalReq) -> Result<u64, AppError> {
    let result = sqlx::query(
        r#"
        INSERT INTO hospital (
            hospital_name, hospital_code, province, city, address, sort_no, status
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(req.hospital_name.trim())
    .bind(trim_optional(&req.hospital_code))
    .bind(trim_optional(&req.province))
    .bind(trim_optional(&req.city))
    .bind(trim_optional(&req.address))
    .bind(req.sort_no.unwrap_or(0))
    .bind(req.status.unwrap_or(1))
    .execute(db)
    .await?;

    Ok(result.last_insert_id())
}

/// 按条件查询数据库记录。
pub async fn find_hospital_by_id(db: &MySqlPool, id: u64) -> Result<Option<Hospital>, AppError> {
    Ok(sqlx::query_as::<_, Hospital>(
        r#"
        SELECT
            id, hospital_name, hospital_code, province, city, address,
            sort_no, status, created_at, updated_at
        FROM hospital
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?)
}

/// 更新业务数据，并在目标不存在时返回 NotFound。
pub async fn update_hospital(
    db: &MySqlPool,
    id: u64,
    req: &SaveHospitalReq,
) -> Result<bool, AppError> {
    let result = sqlx::query(
        r#"
        UPDATE hospital
        SET
            hospital_name = ?, hospital_code = ?, province = ?, city = ?,
            address = ?, sort_no = ?, status = ?
        WHERE id = ?
        "#,
    )
    .bind(req.hospital_name.trim())
    .bind(trim_optional(&req.hospital_code))
    .bind(trim_optional(&req.province))
    .bind(trim_optional(&req.city))
    .bind(trim_optional(&req.address))
    .bind(req.sort_no.unwrap_or(0))
    .bind(req.status.unwrap_or(1))
    .bind(id)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// 从 hospital 表物理删除一条医院记录。
pub async fn delete_hospital(db: &MySqlPool, id: u64) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM hospital WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// 分页查询数据，并返回统一 Page 结构。
pub async fn page_hospitals(
    db: &MySqlPool,
    query: HospitalPageQuery,
) -> Result<Page<Hospital>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let size = query.size.unwrap_or(10).clamp(1, 200);
    let offset = (page - 1) * size;

    let mut data_query = QueryBuilder::<MySql>::new(
        "SELECT id, hospital_name, hospital_code, province, city, address, \
         sort_no, status, created_at, updated_at FROM hospital WHERE 1 = 1",
    );
    push_hospital_filters(&mut data_query, &query);
    data_query.push(" ORDER BY sort_no ASC, id DESC LIMIT ");
    data_query.push_bind(size);
    data_query.push(" OFFSET ");
    data_query.push_bind(offset);

    let records = data_query
        .build_query_as::<Hospital>()
        .fetch_all(db)
        .await?;

    let mut count_query =
        QueryBuilder::<MySql>::new("SELECT COUNT(*) AS total FROM hospital WHERE 1 = 1");
    push_hospital_filters(&mut count_query, &query);
    let total: i64 = count_query.build().fetch_one(db).await?.try_get("total")?;
    let total = total.max(0) as u64;

    Ok(Page {
        records,
        total,
        size,
        current: page,
        pages: if total == 0 {
            0
        } else {
            (total + size - 1) / size
        },
    })
}

/// 向动态 SQL 追加查询条件。
fn push_hospital_filters(query_builder: &mut QueryBuilder<MySql>, query: &HospitalPageQuery) {
    if let Some(hospital_name) = query
        .hospital_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        query_builder.push(" AND hospital_name LIKE ");
        query_builder.push_bind(format!("%{}%", hospital_name.trim()));
    }
    if let Some(hospital_code) = query
        .hospital_code
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        query_builder.push(" AND hospital_code LIKE ");
        query_builder.push_bind(format!("%{}%", hospital_code.trim()));
    }
    if let Some(province) = query
        .province
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        query_builder.push(" AND province = ");
        query_builder.push_bind(province.trim().to_string());
    }
    if let Some(city) = query
        .city
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        query_builder.push(" AND city = ");
        query_builder.push_bind(city.trim().to_string());
    }
    if let Some(status) = query.status {
        query_builder.push(" AND status = ");
        query_builder.push_bind(status);
    }
}

/// 清理可选字符串，空白内容不写入数据库。
fn trim_optional(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
