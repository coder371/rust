//! قراءة الإعدادات من البيئة (ومن ملف `.env` لو موجود).

use std::env;

/// بيحمّل ملف `.env` جوّه متغيّرات البيئة.
///
/// بينادى مرة واحدة عند بداية البرنامج. غياب الملف مش خطأ —
/// في الإنتاج المتغيّرات بتيجي من البيئة نفسها مش من ملف.
pub fn load() {
    dotenvy::dotenv().ok();
}

/// السر المستخدم في توقيع الـ JWT.
pub fn jwt_secret() -> Result<String, Box<dyn std::error::Error>> {
    let secret = env::var("JWT_SECRET")?;

    if secret.trim().is_empty() {
        return Err("JWT_SECRET فاضي — حط قيمة عشوائية طويلة في ملف .env".into());
    }

    Ok(secret)
}

/// مدة صلاحية التوكن بالثواني. الافتراضي ساعة.
pub fn jwt_ttl_seconds() -> Result<u64, Box<dyn std::error::Error>> {
    match env::var("JWT_TTL_SECONDS") {
        Ok(value) => Ok(value.parse()?),
        Err(_) => Ok(3600),
    }
}
