use crate::common::{page, Page};
use crate::error::AppError;
use crate::file::file_model::{FileObject, FileObjectPageQuery, SaveFileObjectReq};
use crate::utils::string_utils;
use sqlx::{MySql, MySqlPool, QueryBuilder, Row};

/// 向数据库插入记录，并返回新记录 ID。
pub async fn insert_file_object(db: &MySqlPool, req: &SaveFileObjectReq) -> Result<u64, AppError> {
    let result = sqlx::query(
        r#"
        INSERT INTO file_object (
            file_name, file_url, mime_type, file_size, sha256
        )
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(req.file_name.trim())
    .bind(req.file_url.trim())
    .bind(string_utils::trim_optional(&req.mime_type))
    .bind(req.file_size)
    .bind(string_utils::trim_optional(&req.sha256))
    .execute(db)
    .await?;

    Ok(result.last_insert_id())
}

/// 按条件查询数据库记录。
pub async fn find_file_object_by_id(
    db: &MySqlPool,
    id: u64,
) -> Result<Option<FileObject>, AppError> {
    Ok(sqlx::query_as::<_, FileObject>(
        r#"
        SELECT id, file_name, file_url, mime_type, file_size, sha256, created_at
        FROM file_object
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?)
}

/// 分页查询数据，并返回统一 Page 结构。
pub async fn page_file_objects(
    db: &MySqlPool,
    query: FileObjectPageQuery,
) -> Result<Page<FileObject>, AppError> {
    let (page, size, offset) = page::page_params(query.page, query.size);
    let mut data_query = QueryBuilder::<MySql>::new(
        "SELECT id, file_name, file_url, mime_type, file_size, sha256, created_at \
         FROM file_object WHERE 1 = 1",
    );
    push_file_object_filters(&mut data_query, &query);
    data_query.push(" ORDER BY id DESC LIMIT ");
    data_query.push_bind(size);
    data_query.push(" OFFSET ");
    data_query.push_bind(offset);

    let records = data_query
        .build_query_as::<FileObject>()
        .fetch_all(db)
        .await?;

    let mut count_query =
        QueryBuilder::<MySql>::new("SELECT COUNT(*) AS total FROM file_object WHERE 1 = 1");
    push_file_object_filters(&mut count_query, &query);
    let total: i64 = count_query.build().fetch_one(db).await?.try_get("total")?;

    Ok(page::to_page(records, total, page, size))
}

/// 向动态 SQL 追加查询条件。
fn push_file_object_filters(query_builder: &mut QueryBuilder<MySql>, query: &FileObjectPageQuery) {
    if let Some(file_name) = string_utils::not_blank(&query.file_name) {
        query_builder.push(" AND file_name LIKE ");
        query_builder.push_bind(format!("%{}%", file_name));
    }
    if let Some(mime_type) = string_utils::not_blank(&query.mime_type) {
        query_builder.push(" AND mime_type = ");
        query_builder.push_bind(mime_type.to_string());
    }
    if let Some(sha256) = string_utils::not_blank(&query.sha256) {
        query_builder.push(" AND sha256 = ");
        query_builder.push_bind(sha256.to_string());
    }
}

/// 删除未被直播间或用户资料引用的文件记录。
pub async fn delete_unreferenced_file_object(db: &MySqlPool, id: u64) -> Result<bool, AppError> {
    let references: i64 = sqlx::query_scalar(
        r#"
        SELECT
            (SELECT COUNT(*) FROM live_room WHERE cover_file_id = ? AND is_deleted = 0)
          + (SELECT COUNT(*) FROM user_info
             WHERE header_id = ?
                OR doctor_cert_file_id = ?
                OR id_card_front_file_id = ?
                OR id_card_back_file_id = ?)
        "#,
    )
    .bind(id)
    .bind(id)
    .bind(id)
    .bind(id)
    .bind(id)
    .fetch_one(db)
    .await?;
    if references > 0 {
        return Err(AppError::BadRequest("文件正在被业务数据引用".to_string()));
    }
    Ok(sqlx::query("DELETE FROM file_object WHERE id=?")
        .bind(id)
        .execute(db)
        .await?
        .rows_affected()
        > 0)
}
