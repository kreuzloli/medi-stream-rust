use crate::error::AppError;
use crate::hospital::catalog_model::{Department, Disease, DiseaseDto};
use sqlx::{MySql, MySqlPool, QueryBuilder};
use std::collections::BTreeMap;

pub async fn find_active_departments(db: &MySqlPool) -> Result<Vec<Department>, AppError> {
    Ok(sqlx::query_as::<_, Department>(
        r#"
        SELECT id, dept_name, dept_code, sort_no
        FROM department
        WHERE status = 1
        ORDER BY sort_no ASC, id ASC
        "#,
    )
    .fetch_all(db)
    .await?)
}

pub async fn find_active_diseases_by_department(
    db: &MySqlPool,
    dept_id: u64,
) -> Result<Vec<Disease>, AppError> {
    Ok(sqlx::query_as::<_, Disease>(
        r#"
        SELECT id, dept_id, disease_name, disease_code, keywords, sort_no, status
        FROM disease
        WHERE dept_id = ? AND status = 1
        ORDER BY sort_no ASC, id ASC
        "#,
    )
    .bind(dept_id)
    .fetch_all(db)
    .await?)
}

pub async fn find_active_disease_map_by_departments(
    db: &MySqlPool,
    dept_ids: &[u64],
) -> Result<BTreeMap<u64, Vec<DiseaseDto>>, AppError> {
    // IN 参数数量是动态的，所以这里用 QueryBuilder 逐个 push_bind。
    let mut query = QueryBuilder::<MySql>::new(
        "SELECT id, dept_id, disease_name, disease_code, keywords, sort_no, status \
         FROM disease WHERE status = 1 AND dept_id IN (",
    );
    let mut separated = query.separated(", ");
    for dept_id in dept_ids {
        separated.push_bind(dept_id);
    }
    query.push(") ORDER BY dept_id ASC, sort_no ASC, id ASC");

    let diseases = query.build_query_as::<Disease>().fetch_all(db).await?;
    let mut result: BTreeMap<u64, Vec<DiseaseDto>> = BTreeMap::new();
    for disease in diseases {
        result
            .entry(disease.dept_id)
            .or_default()
            .push(DiseaseDto::from(disease));
    }
    Ok(result)
}
