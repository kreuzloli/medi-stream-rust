use crate::common::Page;
use crate::error::AppError;
use crate::file::file_model::{FileObject, FileObjectPageQuery, SaveFileObjectReq};
use crate::file::file_repository;
use crate::file::file_storage::{self, StoredFile};
use crate::state::AppState;

/// 校验文件记录保存请求。
///
/// 这个方法主要用于手动创建文件记录的接口：
///
/// POST /files
///
/// 上传接口不需要前端填写这些字段，上传完成后会由服务端自动构造。
pub fn validate_save_file_object_req(req: &SaveFileObjectReq) -> Result<(), AppError> {
    if req.file_name.trim().is_empty() {
        return Err(AppError::BadRequest("文件名称不能为空".to_string()));
    }

    if req.file_url.trim().is_empty() {
        return Err(AppError::BadRequest("文件URL不能为空".to_string()));
    }

    if let Some(sha256) = req.sha256.as_deref() {
        let sha256 = sha256.trim();

        if !sha256.is_empty() && sha256.len() != 64 {
            return Err(AppError::BadRequest("sha256长度必须是64位".to_string()));
        }

        if !sha256.is_empty()
            && !sha256
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        {
            return Err(AppError::BadRequest("sha256格式不正确".to_string()));
        }
    }

    Ok(())
}

/// 创建文件记录。
///
/// 这个方法只负责写入 file_object 表，
/// 不负责把文件写入磁盘。
///
/// 可以用于：
///
/// - 登记外部文件 URL；
/// - 登记云存储文件；
/// - 上传接口完成落盘后的数据库写入。
pub async fn create_file_object(
    state: &AppState,
    req: SaveFileObjectReq,
) -> Result<FileObject, AppError> {
    validate_save_file_object_req(&req)?;

    tracing::info!(
        file_name = %req.file_name.trim(),
        file_url = %req.file_url.trim(),
        mime_type = ?req.mime_type,
        file_size = ?req.file_size,
        "create file object request validated"
    );

    let id = file_repository::insert_file_object(&state.db, &req).await?;

    tracing::info!(file_id = id, "file object inserted");

    file_repository::find_file_object_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::Internal("文件记录创建后无法读取".to_string()))
}

/// 将已经写入本地磁盘的文件保存到 file_object 表。
///
/// 文件落盘发生在 file_storage::save_uploaded_file 中。
///
/// 如果数据库写入失败，这里会删除已经落盘的文件，
/// 避免产生只有磁盘文件、没有数据库记录的孤立文件。
pub async fn create_uploaded_file_object(
    state: &AppState,
    stored_file: StoredFile,
) -> Result<FileObject, AppError> {
    let req = SaveFileObjectReq {
        file_name: stored_file.original_name.clone(),
        file_url: stored_file.public_url.clone(),
        mime_type: Some(stored_file.mime_type.clone()),
        file_size: Some(stored_file.file_size),
        sha256: Some(stored_file.sha256.clone()),
    };

    match create_file_object(state, req).await {
        Ok(file_object) => {
            tracing::info!(
                file_id = file_object.id,
                file_name = %file_object.file_name,
                file_url = %file_object.file_url,
                file_size = ?file_object.file_size,
                "uploaded file object created"
            );
            Ok(file_object)
        }
        Err(error) => {
            tracing::error!(
                path = %stored_file.absolute_path.display(),
                error = %error,
                "database insert failed, rollback uploaded file"
            );
            file_storage::rollback_stored_file(&stored_file.absolute_path).await;
            Err(error)
        }
    }
}

/// 根据 ID 查询文件记录。
pub async fn get_file_object(state: &AppState, id: u64) -> Result<Option<FileObject>, AppError> {
    if id == 0 {
        return Err(AppError::BadRequest("文件ID必须大于0".to_string()));
    }

    file_repository::find_file_object_by_id(&state.db, id).await
}

/// 分页查询文件记录。
pub async fn page_file_objects(
    state: &AppState,
    query: FileObjectPageQuery,
) -> Result<Page<FileObject>, AppError> {
    file_repository::page_file_objects(&state.db, query).await
}

/// 删除文件记录，并尝试删除对应的本地磁盘文件。
///
/// 执行顺序：
///
/// 1. 查询文件记录；
/// 2. Repository 检查文件是否被直播间或用户资料引用；
/// 3. 删除数据库记录；
/// 4. 如果属于本地上传文件，则删除磁盘文件。
///
/// 磁盘删除失败时，只记录错误，不把整个接口判定为失败。
///
/// 原因是数据库记录已经删除，此时前端再次重试也无法恢复。
/// 后续可以通过孤立文件清理任务处理残留文件。
pub async fn delete_file_object(state: &AppState, id: u64) -> Result<bool, AppError> {
    if id == 0 {
        return Err(AppError::BadRequest("文件ID必须大于0".to_string()));
    }

    let Some(file_object) = file_repository::find_file_object_by_id(&state.db, id).await? else {
        return Ok(false);
    };

    // Repository 内部负责检查：
    //
    // live_room.cover_file_id 或 user_info 的头像、证件字段是否仍然引用当前文件。
    let deleted = file_repository::delete_unreferenced_file_object(&state.db, id).await?;

    if !deleted {
        return Ok(false);
    }

    match file_storage::delete_public_file(&file_object.file_url, &state.file_storage).await {
        Ok(local_deleted) => {
            tracing::info!(
                file_id = id,
                file_url = %file_object.file_url,
                local_deleted,
                "file object deleted"
            );
        }

        Err(error) => {
            tracing::error!(
                file_id = id,
                file_url = %file_object.file_url,
                error = %error,
                "file record deleted but local file cleanup failed"
            );
        }
    }

    Ok(true)
}
