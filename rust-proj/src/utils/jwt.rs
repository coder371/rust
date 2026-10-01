//! إنشاء توكن JWT والتحقق منه.

use jsonwebtoken::{
    decode, encode, get_current_timestamp, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};

/// محتوى التوكن (payload).
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// صاحب التوكن — عادةً الـ user id.
    pub sub: String,
    /// وقت الانتهاء، ثوانٍ من epoch.
    pub exp: u64,
}

/// بينشئ توكن صالح لمدة `ttl_seconds` من دلوقتي.
pub fn create_token(
    user_id: &str,
    secret: &[u8],
    ttl_seconds: u64,
) -> Result<String, Box<dyn std::error::Error>> {
    let claims = Claims {
        sub: user_id.to_string(),
        exp: get_current_timestamp() + ttl_seconds,
    };

    let key = EncodingKey::from_secret(secret);

    let token = encode(&Header::default(), &claims, &key)?;

    Ok(token)
}

/// بيتحقق من التوقيع ومن `exp`، وبيرجّع المحتوى.
///
/// `Validation::default()` بتستخدم HS256 وبترفض التوكن المنتهي تلقائياً.
pub fn verify_token(
    token: &str,
    secret: &[u8],
) -> Result<Claims, Box<dyn std::error::Error>> {
    let key = DecodingKey::from_secret(secret);

    let data = decode::<Claims>(token, &key, &Validation::default())?;

    Ok(data.claims)
}
