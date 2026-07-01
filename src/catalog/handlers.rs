use crate::catalog::catalog_model::{DepartmentQuery, DepartmentWithDiseasesDto, DiseaseDto};
use crate::catalog::catalog_service;
use crate::error::AppError;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use redis::AsyncCommands;

const FULL_DEPARTMENT_CACHE_KEY: &str = "full_department";
const CACHE_SECONDS: u64 = 128 * 60 * 60;

pub async fn departments(
    State(state): State<AppState>,
    Query(query): Query<DepartmentQuery>,
) -> Result<Json<Vec<DepartmentWithDiseasesDto>>, AppError> {
    // Query<T> 会把 ?includeDiseases=true 解析成 DepartmentQuery。
    Ok(Json(
        catalog_service::list_departments(&state, query.include_diseases.unwrap_or(false)).await?,
    ))
}

pub async fn diseases_by_department(
    State(state): State<AppState>,
    Path(dept_id): Path<u64>,
) -> Result<Json<Vec<DiseaseDto>>, AppError> {
    Ok(Json(
        catalog_service::list_diseases_by_department(&state, dept_id).await?,
    ))
}

pub async fn full_catalog(
    State(mut state): State<AppState>,
) -> Result<Json<Vec<DepartmentWithDiseasesDto>>, AppError> {
    // full catalog 先读 Redis。缓存坏了就删除并重建，保持和 Java 逻辑一致。
    if let Some(redis) = state.redis.as_mut() {
        let cached: Option<String> = redis.get(FULL_DEPARTMENT_CACHE_KEY).await?;
        if let Some(cached) = cached {
            match serde_json::from_str::<Vec<DepartmentWithDiseasesDto>>(&cached) {
                Ok(value) => return Ok(Json(value)),
                Err(_) => {
                    let _: () = redis.del(FULL_DEPARTMENT_CACHE_KEY).await?;
                }
            }
        }
    }

    let mut catalog = catalog_service::list_departments(&state, true).await?;
    for department in &mut catalog {
        // &mut 表示可变借用：不复制整个 department，只在原对象上补 diseasesPreview。
        department.join_disease_names_ellipsis();
    }

    if let Some(redis) = state.redis.as_mut() {
        let json = serde_json::to_string(&catalog)?;
        let _: () = redis
            .set_ex(FULL_DEPARTMENT_CACHE_KEY, json, CACHE_SECONDS)
            .await?;
    }

    Ok(Json(catalog))
}
