use crate::common::constants::status::{STATUS_DISABLED, STATUS_ENABLED};
use crate::common::validation::validate_enabled_or_disabled;
use crate::common::Page;
use crate::error::AppError;
use crate::live::live_model::{
    FileObject, FileObjectPageQuery, LiveRoom, LiveRoomDetail, LiveRoomPageQuery, LiveRoomStream,
    LiveRoomStreamPageQuery, SaveFileObjectReq, SaveLiveRoomReq, SaveLiveRoomStreamReq,
};
use crate::live::live_repository;
use crate::state::AppState;

const ROOM_STATUS_BANNED: i32 = 2;

/// 校验附件保存请求，确保文件名、URL 和 sha256 合法。
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
    }

    Ok(())
}

/// 校验直播房间保存请求，确保所有者唯一且基础字段合法。
pub fn validate_save_live_room_req(req: &SaveLiveRoomReq) -> Result<(), AppError> {
    let has_user_owner = req.owner_user_id.is_some();
    let has_admin_owner = req.owner_admin_id.is_some();
    if has_user_owner == has_admin_owner {
        return Err(AppError::BadRequest(
            "房主必须且只能指定一个普通用户或管理员".to_string(),
        ));
    }
    if req.owner_user_id == Some(0) || req.owner_admin_id == Some(0) {
        return Err(AppError::BadRequest("房主ID必须大于0".to_string()));
    }
    if req.room_code.trim().is_empty() {
        return Err(AppError::BadRequest("房间编码不能为空".to_string()));
    }
    if req.title.trim().is_empty() {
        return Err(AppError::BadRequest("房间标题不能为空".to_string()));
    }
    validate_enabled_or_disabled(req.is_top, "置顶标记只能是0或1")?;
    validate_room_status(req.status)
}

/// 校验所有者和直播间分类引用，提前返回比数据库外键更明确的业务错误。
async fn validate_live_room_references(
    state: &AppState,
    req: &SaveLiveRoomReq,
) -> Result<(), AppError> {
    if let Some(owner_user_id) = req.owner_user_id {
        if !live_repository::exists_active_user(&state.db, owner_user_id).await? {
            return Err(AppError::BadRequest(
                "房主用户不存在、已停用或已删除".to_string(),
            ));
        }
    }
    if let Some(owner_admin_id) = req.owner_admin_id {
        if !live_repository::exists_active_administrator(&state.db, owner_admin_id).await? {
            return Err(AppError::BadRequest(
                "房主管理员不存在、已停用或已删除".to_string(),
            ));
        }
    }
    if let Some(department_id) = req.department_id {
        if !live_repository::exists_department(&state.db, department_id).await? {
            return Err(AppError::BadRequest("直播间科室不存在".to_string()));
        }
    }
    if let Some(disease_id) = req.disease_id {
        if !live_repository::exists_disease(&state.db, disease_id, req.department_id).await? {
            let message = if req.department_id.is_some() {
                "直播间疾病不存在或不属于所选科室"
            } else {
                "直播间疾病不存在"
            };
            return Err(AppError::BadRequest(message.to_string()));
        }
    }

    Ok(())
}

/// 执行直播间完整校验，并在失败时记录可用于排查的业务标识。
async fn validate_live_room_for_persistence(
    state: &AppState,
    room_id: Option<u64>,
    req: &SaveLiveRoomReq,
) -> Result<(), AppError> {
    let result = match validate_save_live_room_req(req) {
        Ok(()) => validate_live_room_references(state, req).await,
        Err(error) => Err(error),
    };

    if let Err(error) = &result {
        tracing::warn!(
            room_id = ?room_id,
            owner_user_id = ?req.owner_user_id,
            owner_admin_id = ?req.owner_admin_id,
            department_id = ?req.department_id,
            disease_id = ?req.disease_id,
            error = %error,
            "live room validation failed"
        );
    }

    result
}

/// 校验直播流保存请求，确保房间、流编码、streamName 和标记合法。
pub fn validate_save_live_room_stream_req(req: &SaveLiveRoomStreamReq) -> Result<(), AppError> {
    if req.room_id == 0 {
        return Err(AppError::BadRequest("直播房间不能为空".to_string()));
    }
    if req.stream_code.trim().is_empty() {
        return Err(AppError::BadRequest("流编码不能为空".to_string()));
    }
    if req.stream_name.trim().is_empty() {
        return Err(AppError::BadRequest("streamName不能为空".to_string()));
    }
    validate_enabled_or_disabled(req.is_default, "默认流标记只能是0或1")?;
    validate_enabled_or_disabled(req.status, "状态只能是0或1")
}

/// 把直播房间和该房间下的多路流组装成详情响应。
pub fn build_live_room_detail(room: LiveRoom, streams: Vec<LiveRoomStream>) -> LiveRoomDetail {
    LiveRoomDetail { room, streams }
}

/// 创建业务数据，并返回创建后的记录。
pub async fn create_file_object(
    state: &AppState,
    req: SaveFileObjectReq,
) -> Result<FileObject, AppError> {
    validate_save_file_object_req(&req)?;
    tracing::info!(
        file_name = %req.file_name.trim(),
        mime_type = ?req.mime_type,
        file_size = ?req.file_size,
        "create_file_object request validated"
    );
    let id = live_repository::insert_file_object(&state.db, &req).await?;
    tracing::info!(file_id = id, "create_file_object inserted");
    live_repository::find_file_object_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("file object not found".to_string()))
}

/// 根据附件 ID 查询附件记录。
pub async fn get_file_object(state: &AppState, id: u64) -> Result<Option<FileObject>, AppError> {
    live_repository::find_file_object_by_id(&state.db, id).await
}

/// 分页查询数据，并返回统一 Page 结构。
pub async fn page_file_objects(
    state: &AppState,
    query: FileObjectPageQuery,
) -> Result<Page<FileObject>, AppError> {
    live_repository::page_file_objects(&state.db, query).await
}

/// 创建业务数据，并返回创建后的记录。
pub async fn create_live_room(
    state: &AppState,
    req: SaveLiveRoomReq,
) -> Result<LiveRoom, AppError> {
    validate_live_room_for_persistence(state, None, &req).await?;
    tracing::info!(
        owner_user_id = ?req.owner_user_id,
        owner_admin_id = ?req.owner_admin_id,
        department_id = ?req.department_id,
        disease_id = ?req.disease_id,
        is_top = req.is_top.unwrap_or(0),
        room_code = %req.room_code.trim(),
        title = %req.title.trim(),
        "create_live_room request validated"
    );
    let id = live_repository::insert_live_room(&state.db, &req).await?;
    tracing::info!(room_id = id, "create_live_room inserted");
    live_repository::find_live_room_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("live room not found".to_string()))
}

/// 根据房间 ID 查询直播房间主表记录。
pub async fn get_live_room(state: &AppState, id: u64) -> Result<Option<LiveRoom>, AppError> {
    live_repository::find_live_room_by_id(&state.db, id).await
}

/// 查询直播房间详情，并带出该房间下的多路直播流。
pub async fn get_live_room_detail(
    state: &AppState,
    id: u64,
) -> Result<Option<LiveRoomDetail>, AppError> {
    let Some(room) = live_repository::find_live_room_by_id(&state.db, id).await? else {
        tracing::info!(room_id = id, "get_live_room_detail not found");
        return Ok(None);
    };
    let streams = live_repository::list_live_room_streams_by_room_id(&state.db, id).await?;
    tracing::info!(
        room_id = id,
        stream_count = streams.len(),
        "get_live_room_detail loaded streams"
    );

    Ok(Some(build_live_room_detail(room, streams)))
}

/// 更新业务数据，并在目标不存在时返回 NotFound。
pub async fn update_live_room(
    state: &AppState,
    id: u64,
    req: SaveLiveRoomReq,
) -> Result<LiveRoom, AppError> {
    validate_live_room_for_persistence(state, Some(id), &req).await?;
    tracing::info!(
        room_id = id,
        owner_user_id = ?req.owner_user_id,
        owner_admin_id = ?req.owner_admin_id,
        department_id = ?req.department_id,
        disease_id = ?req.disease_id,
        is_top = req.is_top.unwrap_or(0),
        room_code = %req.room_code.trim(),
        "update_live_room request validated"
    );
    let ok = live_repository::update_live_room(&state.db, id, &req).await?;
    if !ok {
        tracing::info!(room_id = id, "update_live_room target not found");
        return Err(AppError::NotFound("live room not found".to_string()));
    }
    tracing::info!(room_id = id, "update_live_room updated");
    live_repository::find_live_room_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("live room not found".to_string()))
}

/// 软删除直播房间，并在记录不存在时返回 NotFound。
pub async fn delete_live_room(state: &AppState, id: u64) -> Result<bool, AppError> {
    let ok = live_repository::soft_delete_live_room(&state.db, id).await?;
    if !ok {
        tracing::info!(room_id = id, "delete_live_room target not found");
        return Err(AppError::NotFound("live room not found".to_string()));
    }
    tracing::info!(room_id = id, "delete_live_room soft deleted");
    Ok(ok)
}

/// 分页查询数据，并返回统一 Page 结构。
pub async fn page_live_rooms(
    state: &AppState,
    query: LiveRoomPageQuery,
) -> Result<Page<LiveRoom>, AppError> {
    live_repository::page_live_rooms(&state.db, query).await
}

/// 创建业务数据，并返回创建后的记录。
pub async fn create_live_room_stream(
    state: &AppState,
    req: SaveLiveRoomStreamReq,
) -> Result<LiveRoomStream, AppError> {
    validate_save_live_room_stream_req(&req)?;
    tracing::info!(
        room_id = req.room_id,
        stream_code = %req.stream_code.trim(),
        stream_name = %req.stream_name.trim(),
        "create_live_room_stream request validated"
    );
    let id = live_repository::insert_live_room_stream(&state.db, &req).await?;
    tracing::info!(
        stream_id = id,
        room_id = req.room_id,
        "create_live_room_stream inserted"
    );
    live_repository::find_live_room_stream_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("live room stream not found".to_string()))
}

/// 根据直播流 ID 查询单条直播流。
pub async fn get_live_room_stream(
    state: &AppState,
    id: u64,
) -> Result<Option<LiveRoomStream>, AppError> {
    live_repository::find_live_room_stream_by_id(&state.db, id).await
}

/// 更新业务数据，并在目标不存在时返回 NotFound。
pub async fn update_live_room_stream(
    state: &AppState,
    id: u64,
    req: SaveLiveRoomStreamReq,
) -> Result<LiveRoomStream, AppError> {
    validate_save_live_room_stream_req(&req)?;
    tracing::info!(
        stream_id = id,
        room_id = req.room_id,
        stream_code = %req.stream_code.trim(),
        "update_live_room_stream request validated"
    );
    let ok = live_repository::update_live_room_stream(&state.db, id, &req).await?;
    if !ok {
        tracing::info!(stream_id = id, "update_live_room_stream target not found");
        return Err(AppError::NotFound("live room stream not found".to_string()));
    }
    tracing::info!(stream_id = id, "update_live_room_stream updated");
    live_repository::find_live_room_stream_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("live room stream not found".to_string()))
}

/// 软删除直播房间，并在记录不存在时返回 NotFound。
pub async fn delete_live_room_stream(state: &AppState, id: u64) -> Result<bool, AppError> {
    let ok = live_repository::soft_delete_live_room_stream(&state.db, id).await?;
    if !ok {
        tracing::info!(stream_id = id, "delete_live_room_stream target not found");
        return Err(AppError::NotFound("live room stream not found".to_string()));
    }
    tracing::info!(stream_id = id, "delete_live_room_stream soft deleted");
    Ok(ok)
}

/// 分页查询数据，并返回统一 Page 结构。
pub async fn page_live_room_streams(
    state: &AppState,
    query: LiveRoomStreamPageQuery,
) -> Result<Page<LiveRoomStream>, AppError> {
    live_repository::page_live_room_streams(&state.db, query).await
}

/// 校验直播房间状态，允许正常、停用和封禁。
fn validate_room_status(status: Option<i32>) -> Result<(), AppError> {
    if let Some(status) = status {
        if !matches!(
            status,
            STATUS_DISABLED | STATUS_ENABLED | ROOM_STATUS_BANNED
        ) {
            return Err(AppError::BadRequest("房间状态只能是0、1或2".to_string()));
        }
    }
    Ok(())
}
