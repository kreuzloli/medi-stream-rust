use crate::account::account_service;
use crate::common::jwt::CurrentUser;
use crate::common::Page;
use crate::error::AppError;
use crate::file::file_model::{FileObject, FileObjectPageQuery, SaveFileObjectReq};
use crate::file::{file_service, file_storage};
use crate::state::AppState;
use axum::extract::{Multipart, Path, Query, State};
use axum::Json;

pub async fn page_files(
    State(state): State<AppState>,
    Query(query): Query<FileObjectPageQuery>,
) -> Result<Json<Page<FileObject>>, AppError> {
    Ok(Json(file_service::page_file_objects(&state, query).await?))
}

pub async fn get_file(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<FileObject>, AppError> {
    Ok(Json(
        file_service::get_file_object(&state, id)
            .await?
            .ok_or_else(|| AppError::NotFound("文件不存在".to_string()))?,
    ))
}

pub async fn create_file(
    State(state): State<AppState>,
    Json(req): Json<SaveFileObjectReq>,
) -> Result<Json<FileObject>, AppError> {
    let file = file_service::create_file_object(&state, req).await?;
    tracing::info!(file_id = file.id, "file object created");
    Ok(Json(file))
}

/// 删除未被直播间引用的文件对象。
pub async fn delete_file(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<bool>, AppError> {
    let deleted = file_service::delete_file_object(&state, id).await?;
    tracing::info!(file_id = id, deleted, "file object deleted");
    Ok(Json(deleted))
}

/// 接收 Multipart 文件，写入本地磁盘并自动创建 file_object 记录。
pub async fn upload_file(
    CurrentUser(claims): CurrentUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<FileObject>, AppError> {
    let user_id = account_service::require_claim_user_id(&claims)?;
    tracing::info!(user_id, "user file upload request received");

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("解析上传请求失败：{e}")))?
    {
        if field.name() != Some("file") {
            continue;
        }
        if field.file_name().is_none() {
            continue;
        }
        // 保存文件
        let stored_file = file_storage::save_uploaded_file(field, &state.file_storage).await?;
        // 写入 file 记录
        let file_obj = file_service::create_uploaded_file_object(&state, stored_file).await?;
        tracing::info!(
            user_id,
            file_id = file_obj.id,
            file_name = %file_obj.file_name,
            file_url = %file_obj.file_url,
            file_size = ?file_obj.file_size,
            "local file uploaded"
        );

        return Ok(Json(file_obj));
    }
    tracing::warn!(user_id, "user file upload request missing file field");
    Err(AppError::BadRequest(
        "上传请求中缺少 file 文件字段".to_string(),
    ))
}
