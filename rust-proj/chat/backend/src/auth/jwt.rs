//! إنشاء توكن JWT والتحقق منه.

use jsonwebtoken::{
    decode, encode, get_current_timestamp, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// اسم المستخدم — هوية صاحب التوكن.
    pub sub: String,
    /// وقت الانتهاء، ثوانٍ من epoch.
    pub exp: u64,
}

pub fn create_token(username: &str, secret: &[u8], ttl_seconds: u64) -> AppResult<String> {
    let claims = Claims {
        sub: username.to_string(),
        exp: get_current_timestamp() + ttl_seconds,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
        .map_err(|error| AppError::Internal(format!("token encoding failed: {error}")))
}

/// بيتحقق من التوقيع ومن `exp`. أي فشل = توكن مرفوض.
pub fn verify_token(token: &str, secret: &[u8]) -> AppResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized("توكن غير صالح أو منتهي".to_string()))
}
