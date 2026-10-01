//! نوع خطأ واحد للتطبيق، بيتحوّل لرد HTTP تلقائياً.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    Conflict(String),
    Internal(String),
}

impl AppError {
    fn parts(&self) -> (StatusCode, &str) {
        match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message.as_str()),
            Self::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message.as_str()),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, message.as_str()),
            Self::Conflict(message) => (StatusCode::CONFLICT, message.as_str()),
            Self::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message.as_str()),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = self.parts();

        // الأخطاء الداخلية بتتسجّل بتفاصيلها، بس العميل بياخد رسالة عامة —
        // عشان ما نسرّبش تفاصيل الخادم.
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!(error = message, "internal error");
            return (status, Json(json!({ "error": "حصل خطأ في الخادم" }))).into_response();
        }

        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
