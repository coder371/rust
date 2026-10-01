//! تجزئة الباسورد والتحقق منه باستخدام Argon2id.

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

/// بيحوّل الباسورد لهاش بصيغة PHC جاهز للتخزين في الداتابيز.
///
/// الـ salt العشوائي بيتولّد داخلياً وبيتخزّن جوّه نفس السلسلة،
/// عشان كده مش محتاج تحتفظ بيه في عمود منفصل.
pub fn hash_password(password: &str) -> Result<String, Box<dyn std::error::Error>> {
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes())?
        .to_string();

    Ok(password_hash)
}

/// بيقارن باسورد خام بهاش مخزّن.
///
/// بيرجّع `Ok(false)` لو الباسورد غلط، و `Err` لو الهاش نفسه تالف/مش مقروء —
/// وده فرق مهم: الأولى محاولة دخول فاشلة، والتانية باج في البيانات.
pub fn verify_password(
    password: &str,
    password_hash: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let parsed_hash = PasswordHash::new(password_hash)?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
