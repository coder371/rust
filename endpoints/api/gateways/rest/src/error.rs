use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use kernel::DomainError;
use serde::Serialize;

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

/// نفس فكرة `DomainResultExt` بتاعة جراف — بس بشكل HTTP.
pub struct RestError(pub DomainError);

impl From<DomainError> for RestError {
    fn from(value: DomainError) -> Self {
        Self(value)
    }
}

impl IntoResponse for RestError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            DomainError::NotFound(_) => StatusCode::NOT_FOUND,
            DomainError::Unauthorized => StatusCode::UNAUTHORIZED,
            DomainError::Forbidden(_) => StatusCode::FORBIDDEN,
            DomainError::Invalid(_) => StatusCode::BAD_REQUEST,
            DomainError::Conflict(_) => StatusCode::CONFLICT,
            DomainError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = ErrorBody {
            code: self.0.code(),
            message: if self.0.is_public() {
                self.0.to_string()
            } else {
                "internal error".to_owned()
            },
        };

        (status, Json(body)).into_response()
    }
}
