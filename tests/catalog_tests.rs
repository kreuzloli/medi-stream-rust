use medi_stream_rust::catalog::catalog_model::{DepartmentWithDiseasesDto, DiseaseDto};

#[test]
fn preview_matches_java_full_catalog_rule() {
    let mut department = DepartmentWithDiseasesDto {
        dept_id: 1,
        dept_name: "心内科".to_string(),
        dept_code: Some("cardiology".to_string()),
        diseases_preview: None,
        sort: 10,
        diseases: vec![
            disease(1, "高血压"),
            disease(2, "冠心病"),
            disease(3, "心律失常"),
        ],
    };

    department.join_disease_names_ellipsis();

    assert_eq!(
        department.diseases_preview.as_deref(),
        Some("高血压 · 冠心病 · ...")
    );
}

fn disease(id: u64, name: &str) -> DiseaseDto {
    DiseaseDto {
        id,
        disease_name: name.to_string(),
        disease_code: None,
        keywords: None,
        sort: id as i32,
        status: 1,
    }
}
