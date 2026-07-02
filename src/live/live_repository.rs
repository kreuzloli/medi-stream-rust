use crate::common::Page;
use crate::error::AppError;
use crate::live::live_model::{
    FileObject, FileObjectPageQuery, LiveRoom, LiveRoomPageQuery, LiveRoomStream,
    LiveRoomStreamPageQuery, SaveFileObjectReq, SaveLiveRoomReq, SaveLiveRoomStreamReq,
};
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
    .bind(trim_optional(&req.mime_type))
    .bind(req.file_size)
    .bind(trim_optional(&req.sha256))
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
    let (page, size, offset) = page_params(query.page, query.size);
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

    Ok(to_page(records, total, page, size))
}

/// 向数据库插入记录，并返回新记录 ID。
pub async fn insert_live_room(db: &MySqlPool, req: &SaveLiveRoomReq) -> Result<u64, AppError> {
    let result = sqlx::query(
        r#"
        INSERT INTO live_room (
            owner_user_id, room_code, title, description, cover_file_id, status
        )
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(req.owner_user_id)
    .bind(req.room_code.trim())
    .bind(req.title.trim())
    .bind(trim_optional(&req.description))
    .bind(req.cover_file_id)
    .bind(req.status.unwrap_or(1))
    .execute(db)
    .await?;

    Ok(result.last_insert_id())
}

/// 按条件查询数据库记录。
pub async fn find_live_room_by_id(db: &MySqlPool, id: u64) -> Result<Option<LiveRoom>, AppError> {
    Ok(sqlx::query_as::<_, LiveRoom>(
        r#"
        SELECT
            id, owner_user_id, room_code, title, description, cover_file_id,
            status, is_deleted, created_at, updated_at
        FROM live_room
        WHERE id = ? AND is_deleted = 0
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?)
}

/// 更新业务数据，并在目标不存在时返回 NotFound。
pub async fn update_live_room(
    db: &MySqlPool,
    id: u64,
    req: &SaveLiveRoomReq,
) -> Result<bool, AppError> {
    let result = sqlx::query(
        r#"
        UPDATE live_room
        SET
            owner_user_id = ?, room_code = ?, title = ?, description = ?,
            cover_file_id = ?, status = ?
        WHERE id = ? AND is_deleted = 0
        "#,
    )
    .bind(req.owner_user_id)
    .bind(req.room_code.trim())
    .bind(req.title.trim())
    .bind(trim_optional(&req.description))
    .bind(req.cover_file_id)
    .bind(req.status.unwrap_or(1))
    .bind(id)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// 执行软删除，只修改 is_deleted 标记。
pub async fn soft_delete_live_room(db: &MySqlPool, id: u64) -> Result<bool, AppError> {
    let result = sqlx::query("UPDATE live_room SET is_deleted = 1 WHERE id = ? AND is_deleted = 0")
        .bind(id)
        .execute(db)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// 分页查询数据，并返回统一 Page 结构。
pub async fn page_live_rooms(
    db: &MySqlPool,
    query: LiveRoomPageQuery,
) -> Result<Page<LiveRoom>, AppError> {
    let (page, size, offset) = page_params(query.page, query.size);
    let mut data_query = QueryBuilder::<MySql>::new(
        "SELECT id, owner_user_id, room_code, title, description, cover_file_id, \
         status, is_deleted, created_at, updated_at FROM live_room WHERE is_deleted = 0",
    );
    push_live_room_filters(&mut data_query, &query);
    data_query.push(" ORDER BY id DESC LIMIT ");
    data_query.push_bind(size);
    data_query.push(" OFFSET ");
    data_query.push_bind(offset);

    let records = data_query
        .build_query_as::<LiveRoom>()
        .fetch_all(db)
        .await?;

    let mut count_query =
        QueryBuilder::<MySql>::new("SELECT COUNT(*) AS total FROM live_room WHERE is_deleted = 0");
    push_live_room_filters(&mut count_query, &query);
    let total: i64 = count_query.build().fetch_one(db).await?.try_get("total")?;

    Ok(to_page(records, total, page, size))
}

/// 向数据库插入记录，并返回新记录 ID。
pub async fn insert_live_room_stream(
    db: &MySqlPool,
    req: &SaveLiveRoomStreamReq,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        r#"
        INSERT INTO live_room_stream (
            room_id, stream_code, stream_name, title, sort_no, is_default, status
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(req.room_id)
    .bind(req.stream_code.trim())
    .bind(req.stream_name.trim())
    .bind(trim_optional(&req.title))
    .bind(req.sort_no.unwrap_or(0))
    .bind(req.is_default.unwrap_or(0))
    .bind(req.status.unwrap_or(1))
    .execute(db)
    .await?;

    Ok(result.last_insert_id())
}

/// 按条件查询数据库记录。
pub async fn find_live_room_stream_by_id(
    db: &MySqlPool,
    id: u64,
) -> Result<Option<LiveRoomStream>, AppError> {
    Ok(sqlx::query_as::<_, LiveRoomStream>(
        r#"
        SELECT
            id, room_id, stream_code, stream_name, title, sort_no, is_default,
            status, is_deleted, created_at, updated_at
        FROM live_room_stream
        WHERE id = ? AND is_deleted = 0
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?)
}

/// 按条件查询列表数据。
pub async fn list_live_room_streams_by_room_id(
    db: &MySqlPool,
    room_id: u64,
) -> Result<Vec<LiveRoomStream>, AppError> {
    // 房间详情需要一次性取出多路流；只返回未删除的流，排序规则和展示顺序保持一致。
    Ok(sqlx::query_as::<_, LiveRoomStream>(
        r#"
        SELECT
            id, room_id, stream_code, stream_name, title, sort_no, is_default,
            status, is_deleted, created_at, updated_at
        FROM live_room_stream
        WHERE room_id = ? AND is_deleted = 0
        ORDER BY sort_no ASC, id ASC
        "#,
    )
    .bind(room_id)
    .fetch_all(db)
    .await?)
}

/// 更新业务数据，并在目标不存在时返回 NotFound。
pub async fn update_live_room_stream(
    db: &MySqlPool,
    id: u64,
    req: &SaveLiveRoomStreamReq,
) -> Result<bool, AppError> {
    let result = sqlx::query(
        r#"
        UPDATE live_room_stream
        SET
            room_id = ?, stream_code = ?, stream_name = ?, title = ?,
            sort_no = ?, is_default = ?, status = ?
        WHERE id = ? AND is_deleted = 0
        "#,
    )
    .bind(req.room_id)
    .bind(req.stream_code.trim())
    .bind(req.stream_name.trim())
    .bind(trim_optional(&req.title))
    .bind(req.sort_no.unwrap_or(0))
    .bind(req.is_default.unwrap_or(0))
    .bind(req.status.unwrap_or(1))
    .bind(id)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// 执行软删除，只修改 is_deleted 标记。
pub async fn soft_delete_live_room_stream(db: &MySqlPool, id: u64) -> Result<bool, AppError> {
    let result =
        sqlx::query("UPDATE live_room_stream SET is_deleted = 1 WHERE id = ? AND is_deleted = 0")
            .bind(id)
            .execute(db)
            .await?;

    Ok(result.rows_affected() > 0)
}

/// 分页查询数据，并返回统一 Page 结构。
pub async fn page_live_room_streams(
    db: &MySqlPool,
    query: LiveRoomStreamPageQuery,
) -> Result<Page<LiveRoomStream>, AppError> {
    let (page, size, offset) = page_params(query.page, query.size);
    let mut data_query = QueryBuilder::<MySql>::new(
        "SELECT id, room_id, stream_code, stream_name, title, sort_no, is_default, \
         status, is_deleted, created_at, updated_at FROM live_room_stream WHERE is_deleted = 0",
    );
    push_live_room_stream_filters(&mut data_query, &query);
    data_query.push(" ORDER BY room_id ASC, sort_no ASC, id ASC LIMIT ");
    data_query.push_bind(size);
    data_query.push(" OFFSET ");
    data_query.push_bind(offset);

    let records = data_query
        .build_query_as::<LiveRoomStream>()
        .fetch_all(db)
        .await?;

    let mut count_query = QueryBuilder::<MySql>::new(
        "SELECT COUNT(*) AS total FROM live_room_stream WHERE is_deleted = 0",
    );
    push_live_room_stream_filters(&mut count_query, &query);
    let total: i64 = count_query.build().fetch_one(db).await?.try_get("total")?;

    Ok(to_page(records, total, page, size))
}

/// 向动态 SQL 追加查询条件。
fn push_file_object_filters(query_builder: &mut QueryBuilder<MySql>, query: &FileObjectPageQuery) {
    if let Some(file_name) = not_blank(&query.file_name) {
        query_builder.push(" AND file_name LIKE ");
        query_builder.push_bind(format!("%{}%", file_name));
    }
    if let Some(mime_type) = not_blank(&query.mime_type) {
        query_builder.push(" AND mime_type = ");
        query_builder.push_bind(mime_type.to_string());
    }
    if let Some(sha256) = not_blank(&query.sha256) {
        query_builder.push(" AND sha256 = ");
        query_builder.push_bind(sha256.to_string());
    }
}

/// 向动态 SQL 追加查询条件。
fn push_live_room_filters(query_builder: &mut QueryBuilder<MySql>, query: &LiveRoomPageQuery) {
    if let Some(owner_user_id) = query.owner_user_id {
        query_builder.push(" AND owner_user_id = ");
        query_builder.push_bind(owner_user_id);
    }
    if let Some(room_code) = not_blank(&query.room_code) {
        query_builder.push(" AND room_code LIKE ");
        query_builder.push_bind(format!("%{}%", room_code));
    }
    if let Some(title) = not_blank(&query.title) {
        query_builder.push(" AND title LIKE ");
        query_builder.push_bind(format!("%{}%", title));
    }
    if let Some(status) = query.status {
        query_builder.push(" AND status = ");
        query_builder.push_bind(status);
    }
}

/// 向动态 SQL 追加查询条件。
fn push_live_room_stream_filters(
    query_builder: &mut QueryBuilder<MySql>,
    query: &LiveRoomStreamPageQuery,
) {
    if let Some(room_id) = query.room_id {
        query_builder.push(" AND room_id = ");
        query_builder.push_bind(room_id);
    }
    if let Some(stream_code) = not_blank(&query.stream_code) {
        query_builder.push(" AND stream_code LIKE ");
        query_builder.push_bind(format!("%{}%", stream_code));
    }
    if let Some(stream_name) = not_blank(&query.stream_name) {
        query_builder.push(" AND stream_name LIKE ");
        query_builder.push_bind(format!("%{}%", stream_name));
    }
    if let Some(status) = query.status {
        query_builder.push(" AND status = ");
        query_builder.push_bind(status);
    }
}

/// 分页查询数据，并返回统一 Page 结构。
fn page_params(page: Option<u64>, size: Option<u64>) -> (u64, u64, u64) {
    let page = page.unwrap_or(1).max(1);
    let size = size.unwrap_or(10).clamp(1, 200);
    let offset = (page - 1) * size;

    (page, size, offset)
}

/// 根据分页参数计算当前页、每页数量和 SQL offset。
fn to_page<T>(records: Vec<T>, total: i64, page: u64, size: u64) -> Page<T> {
    let total = total.max(0) as u64;
    Page {
        records,
        total,
        size,
        current: page,
        pages: if total == 0 {
            0
        } else {
            (total + size - 1) / size
        },
    }
}

/// 判断可选字符串是否为非空查询条件。
fn trim_optional(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

/// 判断可选字符串是否有有效内容。
fn not_blank(value: &Option<String>) -> Option<&str> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}
