//! تجزئة الباسورد بـ Argon2id.
//!
//! Argon2 مصمَّم عمداً إنه بطيء وبيستهلك رام (~19 ميجا للمرة الواحدة)،
//! عشان يبوّظ محاولات التخمين. عشان كده كل نداء بيتحط في `spawn_blocking`:
//! لو اشتغل على خيط async هيوقّف باقي الاتصالات معاه.

use std::sync::OnceLock;

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use tokio::sync::{Semaphore, SemaphorePermit};

use crate::error::{AppError, AppResult};

/// عدد عمليات التجزئة المسموح بيها في نفس اللحظة.
///
/// كل عملية Argon2 بتحجز ~١٩ ميجا. من غير الحد ده، `spawn_blocking`
/// بيوسّع لـ٥١٢ خيط افتراضياً — يعني ٥١٢ تسجيل متزامن يقفوا الخادم
/// بـ~١٠ جيجا رام. الحد بيخلّي الطلبات الزيادة تستنى في طابور بدل
/// ما تتنافس على ذاكرة مش موجودة.
fn hash_slots() -> &'static Semaphore {
    static SLOTS: OnceLock<Semaphore> = OnceLock::new();

    SLOTS.get_or_init(|| {
        let cores = std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(4);

        Semaphore::new(cores)
    })
}

/// بيحجز مكان قبل التجزئة. الانتظار هنا مقصود.
async fn acquire_slot() -> AppResult<SemaphorePermit<'static>> {
    hash_slots()
        .acquire()
        .await
        .map_err(|_| AppError::Internal("hash semaphore closed".to_string()))
}

/// بيحوّل الباسورد لهاش بصيغة PHC جاهز للتخزين.
pub async fn hash_password(password: String) -> AppResult<String> {
    let _slot = acquire_slot().await?;

    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|hash| hash.to_string())
            .map_err(|error| AppError::Internal(format!("hashing failed: {error}")))
    })
    .await
    .map_err(|error| AppError::Internal(format!("join failed: {error}")))?
}

/// بيقارن باسورد خام بهاش مخزّن.
///
/// `Ok(false)` = الباسورد غلط. `Err` = الهاش نفسه تالف، وده باج مش خطأ مستخدم.
pub async fn verify_password(password: String, password_hash: String) -> AppResult<bool> {
    let _slot = acquire_slot().await?;

    tokio::task::spawn_blocking(move || {
        let parsed = PasswordHash::new(&password_hash)
            .map_err(|error| AppError::Internal(format!("stored hash is corrupt: {error}")))?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok())
    })
    .await
    .map_err(|error| AppError::Internal(format!("join failed: {error}")))?
}
