use crate::catalog::model::{DepartmentWithDiseasesDto, DiseaseDto};
use crate::catalog::repository;
use crate::error::AppError;
use crate::state::AppState;

pub async fn list_departments(
    state: &AppState,
    include_diseases: bool,
) -> Result<Vec<DepartmentWithDiseasesDto>, AppError> {
    // 先查启用科室；includeDiseases=false 时不查疾病，减少一次数据库访问。
    let departments = repository::find_active_departments(&state.db).await?;

    if !include_diseases || departments.is_empty() {
        return Ok(departments
            .into_iter()
            .map(|department| DepartmentWithDiseasesDto::from_department(department, Vec::new()))
            .collect());
    }

    let dept_ids = departments
        .iter()
        .map(|department| department.id)
        .collect::<Vec<_>>();
    // 批量查疾病，避免每个科室查一次导致 N+1 查询问题。
    let disease_map =
        repository::find_active_disease_map_by_departments(&state.db, &dept_ids).await?;

    Ok(departments
        .into_iter()
        .map(|department| {
            let diseases = disease_map.get(&department.id).cloned().unwrap_or_default();
            DepartmentWithDiseasesDto::from_department(department, diseases)
        })
        .collect())
}

pub async fn list_diseases_by_department(
    state: &AppState,
    dept_id: u64,
) -> Result<Vec<DiseaseDto>, AppError> {
    let diseases = repository::find_active_diseases_by_department(&state.db, dept_id).await?;
    Ok(diseases.into_iter().map(DiseaseDto::from).collect())
}
