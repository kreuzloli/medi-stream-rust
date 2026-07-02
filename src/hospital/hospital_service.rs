use crate::common::validation::validate_enabled_or_disabled;
use crate::common::Page;
use crate::error::AppError;
use crate::hospital::hospital_model::{Hospital, HospitalPageQuery, SaveHospitalReq};
use crate::hospital::hospital_repository;
use crate::state::AppState;

/// 校验医院保存请求，确保名称和状态合法。
pub fn validate_save_hospital_req(req: &SaveHospitalReq) -> Result<(), AppError> {
    if req.hospital_name.trim().is_empty() {
        return Err(AppError::BadRequest("医院名称不能为空".to_string()));
    }
    validate_enabled_or_disabled(req.status, "状态只能是0或1")
}

/// 创建业务数据，并返回创建后的记录。
pub async fn create_hospital(state: &AppState, req: SaveHospitalReq) -> Result<Hospital, AppError> {
    validate_save_hospital_req(&req)?;
    tracing::info!(
        hospital_name = %req.hospital_name.trim(),
        hospital_code = ?req.hospital_code,
        "create_hospital request validated"
    );
    let id = hospital_repository::insert_hospital(&state.db, &req).await?;
    tracing::info!(hospital_id = id, "create_hospital inserted");
    hospital_repository::find_hospital_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("hospital not found".to_string()))
}

/// 根据医院 ID 查询医院记录。
pub async fn get_hospital(state: &AppState, id: u64) -> Result<Option<Hospital>, AppError> {
    hospital_repository::find_hospital_by_id(&state.db, id).await
}

/// 更新业务数据，并在目标不存在时返回 NotFound。
pub async fn update_hospital(
    state: &AppState,
    id: u64,
    req: SaveHospitalReq,
) -> Result<Hospital, AppError> {
    validate_save_hospital_req(&req)?;
    tracing::info!(
        hospital_id = id,
        hospital_name = %req.hospital_name.trim(),
        "update_hospital request validated"
    );
    let ok = hospital_repository::update_hospital(&state.db, id, &req).await?;
    if !ok {
        tracing::info!(hospital_id = id, "update_hospital target not found");
        return Err(AppError::NotFound("hospital not found".to_string()));
    }
    tracing::info!(hospital_id = id, "update_hospital updated");
    hospital_repository::find_hospital_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("hospital not found".to_string()))
}

/// 删除医院记录，并在记录不存在时返回 NotFound。
pub async fn delete_hospital(state: &AppState, id: u64) -> Result<bool, AppError> {
    let ok = hospital_repository::delete_hospital(&state.db, id).await?;
    if !ok {
        tracing::info!(hospital_id = id, "delete_hospital target not found");
        return Err(AppError::NotFound("hospital not found".to_string()));
    }
    tracing::info!(hospital_id = id, "delete_hospital deleted");
    Ok(ok)
}

/// 分页查询数据，并返回统一 Page 结构。
pub async fn page_hospitals(
    state: &AppState,
    query: HospitalPageQuery,
) -> Result<Page<Hospital>, AppError> {
    hospital_repository::page_hospitals(&state.db, query).await
}
