use medi_stream_rust::hospital::hospital_model::SaveHospitalReq;
use medi_stream_rust::hospital::hospital_service::validate_save_hospital_req;

/// 验证保存请求的核心行为。
#[test]
fn save_hospital_requires_name() {
    let mut req = valid_save_hospital_req();
    req.hospital_name = "  ".to_string();

    let err = validate_save_hospital_req(&req).expect_err("blank hospital name must be rejected");

    assert!(err.to_string().contains("医院名称不能为空"));
}

/// 验证保存请求的核心行为。
#[test]
fn save_hospital_rejects_invalid_status() {
    let req = SaveHospitalReq {
        status: Some(2),
        ..valid_save_hospital_req()
    };

    let err = validate_save_hospital_req(&req).expect_err("status must be 0 or 1");

    assert!(err.to_string().contains("状态只能是0或1"));
}

/// 验证保存请求的核心行为。
#[test]
fn save_hospital_accepts_valid_required_fields() {
    let req = valid_save_hospital_req();

    validate_save_hospital_req(&req).expect("valid hospital request should pass validation");
}

/// 构造测试使用的有效请求对象。
fn valid_save_hospital_req() -> SaveHospitalReq {
    SaveHospitalReq {
        hospital_name: "北京协和医院".to_string(),
        hospital_code: Some("PUMCH".to_string()),
        province: Some("北京".to_string()),
        city: Some("北京".to_string()),
        address: Some("东城区帅府园一号".to_string()),
        sort_no: Some(10),
        status: Some(1),
    }
}
