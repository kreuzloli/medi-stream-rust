use medi_stream_rust::common::validation::validate_enabled_or_disabled;
use medi_stream_rust::error::AppError;

#[test]
fn validate_enabled_or_disabled_accepts_empty_zero_and_one() {
    assert!(validate_enabled_or_disabled(None, "状态只能是0或1").is_ok());
    assert!(validate_enabled_or_disabled(Some(0), "状态只能是0或1").is_ok());
    assert!(validate_enabled_or_disabled(Some(1), "状态只能是0或1").is_ok());
}

#[test]
fn validate_enabled_or_disabled_rejects_other_values_with_message() {
    let err = validate_enabled_or_disabled(Some(2), "默认流标记只能是0或1").unwrap_err();

    assert!(matches!(err, AppError::BadRequest(message) if message == "默认流标记只能是0或1"));
}
