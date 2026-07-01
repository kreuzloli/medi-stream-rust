use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    // thiserror 负责生成 Display/Error 实现；#[from] 可以让 ? 自动转换错误类型。
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    NotFound(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Internal(String),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: u16,
    message: String,
}

impl AppError {
    fn status(&self) -> StatusCode {
        // 把内部错误类型映射成 HTTP 状态码，集中处理比每个 handler 手写响应更稳定。
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) | AppError::Jwt(_) => StatusCode::UNAUTHORIZED,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Database(_)
            | AppError::Redis(_)
            | AppError::Json(_)
            | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Axum 遇到 handler 返回 Err(AppError) 时，会调用这里生成最终 HTTP 响应。
        let status = self.status();
        (
            status,
            Json(ErrorBody {
                code: status.as_u16(),
                message: self.to_string(),
            }),
        )
            .into_response()
    }
}
