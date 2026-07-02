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
    // reqwest 自身错误：比如网络断了、DNS 失败、响应 JSON 解析失败等。
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    // 外部 API 返回了非 2xx 状态码。
    // 比如微信返回 400，腾讯云返回 403，这种不是我们服务内部崩了，
    // 而是外部服务明确告诉我们“请求不对”。
    #[error("external api error, service={service}, status={status}, body={body}")]
    ExternalApi {
        service: String,
        status: u16,
        body: String,
    },
    #[error("{0}")]
    Internal(String),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: u16,
    message: String,
}

impl AppError {
    /// 把业务错误类型映射成对外 HTTP 状态码。
    fn status(&self) -> StatusCode {
        // 把内部错误类型映射成 HTTP 状态码，集中处理比每个 handler 手写响应更稳定。
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) | AppError::Jwt(_) => StatusCode::UNAUTHORIZED,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            // 外部服务调用失败，对前端来说更接近“网关/上游服务失败”。
            AppError::Http(_) | AppError::ExternalApi { .. } => StatusCode::BAD_GATEWAY,
            AppError::Database(_)
            | AppError::Redis(_)
            | AppError::Json(_)
            | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    /// 把 AppError 转换成 Axum 可以直接返回的 JSON 响应。
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
