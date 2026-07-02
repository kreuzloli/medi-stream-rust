use crate::common::Page;
use crate::error::AppError;
use crate::hospital::hospital_model::{Hospital, HospitalPageQuery, SaveHospitalReq};
use crate::hospital::hospital_repository;
use crate::state::AppState;

pub fn validate_save_hospital_req(req: &SaveHospitalReq) -> Result<(), AppError> {
    if req.hospital_name.trim().is_empty() {
        return Err(AppError::BadRequest("医院名称不能为空".to_string()));
    }
    validate_status(req.status)
}

pub async fn create_hospital(state: &AppState, req: SaveHospitalReq) -> Result<Hospital, AppError> {
    validate_save_hospital_req(&req)?;
    let id = hospital_repository::insert_hospital(&state.db, &req).await?;
    hospital_repository::find_hospital_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("hospital not found".to_string()))
}

pub async fn get_hospital(state: &AppState, id: u64) -> Result<Option<Hospital>, AppError> {
    hospital_repository::find_hospital_by_id(&state.db, id).await
}

pub async fn update_hospital(
    state: &AppState,
    id: u64,
    req: SaveHospitalReq,
) -> Result<Hospital, AppError> {
    validate_save_hospital_req(&req)?;
    let ok = hospital_repository::update_hospital(&state.db, id, &req).await?;
    if !ok {
        return Err(AppError::NotFound("hospital not found".to_string()));
    }
    hospital_repository::find_hospital_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("hospital not found".to_string()))
}

pub async fn delete_hospital(state: &AppState, id: u64) -> Result<bool, AppError> {
    let ok = hospital_repository::delete_hospital(&state.db, id).await?;
    if !ok {
        return Err(AppError::NotFound("hospital not found".to_string()));
    }
    Ok(ok)
}

pub async fn page_hospitals(
    state: &AppState,
    query: HospitalPageQuery,
) -> Result<Page<Hospital>, AppError> {
    hospital_repository::page_hospitals(&state.db, query).await
}

fn validate_status(status: Option<i32>) -> Result<(), AppError> {
    if let Some(status) = status {
        if !matches!(status, 0 | 1) {
            return Err(AppError::BadRequest("状态只能是0或1".to_string()));
        }
    }
    Ok(())
}
