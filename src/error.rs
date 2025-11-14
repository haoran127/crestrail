use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// 应用错误类型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("无效的查询参数: {0}")]
    InvalidQuery(String),

    #[error("无效的 JSON 数据: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("未授权: {0}")]
    Unauthorized(String),

    #[error("禁止访问: {0}")]
    Forbidden(String),

    #[error("资源未找到: {0}")]
    NotFound(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(ref e) => {
                tracing::error!("数据库错误: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::InvalidQuery(ref msg) => {
                (StatusCode::BAD_REQUEST, msg.clone())
            }
            AppError::InvalidJson(ref e) => {
                (StatusCode::BAD_REQUEST, format!("JSON 解析错误: {}", e))
            }
            AppError::Unauthorized(ref msg) => {
                (StatusCode::UNAUTHORIZED, msg.clone())
            }
            AppError::Forbidden(ref msg) => {
                (StatusCode::FORBIDDEN, msg.clone())
            }
            AppError::NotFound(ref msg) => {
                (StatusCode::NOT_FOUND, msg.clone())
            }
            AppError::Internal(ref msg) => {
                tracing::error!("内部错误: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

