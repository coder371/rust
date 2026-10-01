pub mod jwt;
pub mod password;
pub mod routes;

use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

use crate::{error::AppError, state::AppState};

/// مستخدم متحقَّق منه.
///
/// أي هاندلر بياخد `AuthUser` كباراميتر بيبقى محمي تلقائياً — لو التوكن
/// ناقص أو باظ، axum بترفض الطلب قبل ما الهاندلر يشتغل أصلاً.
pub struct AuthUser {
    pub username: String,
}

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized("مفيش توكن".to_string()))?;

        let claims = jwt::verify_token(token, state.config.jwt_secret.as_bytes())?;

        Ok(Self {
            username: claims.sub,
        })
    }
}
