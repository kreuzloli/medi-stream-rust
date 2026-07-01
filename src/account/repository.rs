use crate::account::model::{AccountPageQuery, UserInfo};
use crate::common::Page;
use crate::error::AppError;
use sqlx::{MySql, MySqlPool, QueryBuilder, Row};

pub async fn insert_user(db: &MySqlPool, req: &UserInfo) -> Result<u64, AppError> {
    let result = sqlx::query(
        r#"
        INSERT INTO user_info (user_code, nickname, email, phone, status, version, is_deleted)
        VALUES (?, ?, ?, ?, ?, COALESCE(?, 0), COALESCE(?, 0))
        "#,
    )
    .bind(&req.user_code)
    .bind(&req.nickname)
    .bind(&req.email)
    .bind(&req.phone)
    .bind(req.status.unwrap_or(1))
    .bind(req.version)
    .bind(req.is_deleted)
    .execute(db)
    .await?;

    Ok(result.last_insert_id())
}

pub async fn find_user_by_id(db: &MySqlPool, id: u64) -> Result<Option<UserInfo>, AppError> {
    Ok(sqlx::query_as::<_, UserInfo>(
        r#"
        SELECT id, user_code, nickname, email, phone, status, version, is_deleted, created_at, updated_at
        FROM user_info
        WHERE id = ? AND is_deleted = 0
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?)
}

pub async fn update_user(db: &MySqlPool, id: u64, req: &UserInfo) -> Result<bool, AppError> {
    let result = sqlx::query(
        r#"
        UPDATE user_info
        SET user_code = ?, nickname = ?, email = ?, phone = ?, status = ?, version = COALESCE(?, version)
        WHERE id = ? AND is_deleted = 0
        "#,
    )
    .bind(&req.user_code)
    .bind(&req.nickname)
    .bind(&req.email)
    .bind(&req.phone)
    .bind(req.status)
    .bind(req.version)
    .bind(id)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn logical_delete_user(db: &MySqlPool, id: u64) -> Result<bool, AppError> {
    let result = sqlx::query("UPDATE user_info SET is_deleted = 1 WHERE id = ? AND is_deleted = 0")
        .bind(id)
        .execute(db)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn page_users(
    db: &MySqlPool,
    query: AccountPageQuery,
) -> Result<Page<UserInfo>, AppError> {
    // 对分页参数做基本保护，避免 size 过大拖垮接口。
    let page = query.page.unwrap_or(1).max(1);
    let size = query.size.unwrap_or(10).clamp(1, 200);
    let offset = (page - 1) * size;
    let user_code = query.user_code.filter(|value| !value.trim().is_empty());

    // QueryBuilder 用绑定参数拼 SQL，避免手写字符串拼接造成 SQL 注入。
    let mut data_query = QueryBuilder::<MySql>::new(
        "SELECT id, user_code, nickname, email, phone, status, version, is_deleted, created_at, updated_at \
         FROM user_info WHERE is_deleted = 0",
    );
    if let Some(user_code) = &user_code {
        data_query.push(" AND user_code LIKE ");
        data_query.push_bind(format!("%{user_code}%"));
    }
    data_query.push(" ORDER BY id DESC LIMIT ");
    data_query.push_bind(size);
    data_query.push(" OFFSET ");
    data_query.push_bind(offset);

    let records = data_query
        .build_query_as::<UserInfo>()
        .fetch_all(db)
        .await?;

    let mut count_query =
        QueryBuilder::<MySql>::new("SELECT COUNT(*) AS total FROM user_info WHERE is_deleted = 0");
    if let Some(user_code) = &user_code {
        count_query.push(" AND user_code LIKE ");
        count_query.push_bind(format!("%{user_code}%"));
    }
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
