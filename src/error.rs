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

