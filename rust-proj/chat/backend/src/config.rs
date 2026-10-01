//! إعدادات الخادم، مقروءة من البيئة أو من ملف `.env`.

use std::env;

#[derive(Clone)]
pub struct Config {
    /// سر توقيع الـ JWT.
    pub jwt_secret: String,
    /// مدة صلاحية التوكن بالثواني.
    pub jwt_ttl_seconds: u64,
    /// عنوان الاستماع، مثال: `127.0.0.1:3000`.
    pub bind_addr: String,
    /// أصول مسموح لها بالاتصال (الفرونت وقت التطوير).
    pub allowed_origins: Vec<String>,
    /// أقصى عدد رسائل محفوظة لكل غرفة.
    pub history_limit: usize,
    /// أقصى عدد رسائل من اتصال واحد داخل نافذة `rate_window_secs`.
    pub rate_limit: u32,
    /// طول نافذة تحديد المعدّل بالثواني.
    pub rate_window_secs: u64,
    /// أقصى طول للرسالة الواحدة بالحروف.
    pub max_message_chars: usize,
    /// سعة قناة البث لكل غرفة.
    ///
    /// دي المفاضلة الأساسية: السعة الكبيرة بتستحمل الدفعات المفاجئة من
    /// غير فقدان، بس بتاكل ذاكرة لكل غرفة. الصغيرة بتحافظ على الذاكرة
    /// ثابتة وبتضحّي برسائل العميل البطيء.
    pub broadcast_capacity: usize,
}

impl Config {
    /// بيقرا الإعدادات من البيئة. بيفشل بس لو السر ناقص.
    pub fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| "JWT_SECRET غير موجود — انسخ .env.example إلى .env".to_string())?;

        if jwt_secret.trim().is_empty() {
            return Err("JWT_SECRET فاضي — حط قيمة عشوائية طويلة".to_string());
        }

        Ok(Self {
            jwt_secret,
            jwt_ttl_seconds: parse_or("JWT_TTL_SECONDS", 86_400),
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string()),
            allowed_origins: env::var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173".to_string())
                .split(',')
                .map(|origin| origin.trim().to_string())
                .filter(|origin| !origin.is_empty())
                .collect(),
            history_limit: parse_or("HISTORY_LIMIT", 100),
            rate_limit: parse_or("RATE_LIMIT", 15),
            rate_window_secs: parse_or("RATE_WINDOW_SECS", 5),
            max_message_chars: parse_or("MAX_MESSAGE_CHARS", 2_000),
            broadcast_capacity: parse_or("BROADCAST_CAPACITY", 2_048),
        })
    }
}

/// بيقرا متغيّر رقمي، وبيرجّع الافتراضي لو مش موجود أو مش صالح.
fn parse_or<T: std::str::FromStr>(key: &str, default: T) -> T {
    env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}
