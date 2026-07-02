use crate::common::constants::status::is_enabled_or_disabled;
use crate::error::AppError;

/// 校验可选的启用/禁用状态值；为空表示不更新或使用默认值。
pub fn validate_enabled_or_disabled(value: Option<i32>, message: &str) -> Result<(), AppError> {
    if let Some(value) = value {
        if !is_enabled_or_disabled(value) {
            return Err(AppError::BadRequest(message.to_string()));
        }
    }
    Ok(())
}
