//! نفس أعلام نسخة نود بالظبط، عشان المقارنة تبقى على المحرّك مش على الضبط.

use std::env;

#[derive(Clone, Copy)]
pub struct Config {
    /// كاش قوالب القالب — مقابل `noCache` في Nunjucks
    pub tpl_cache: bool,
    /// قراءة كاش الصفحة (المتعلّقة في core-service)
    pub page_cache: bool,
    /// معالجة حقول الويدجت بالتوازي بدل التتابع
    pub parallel_fields: bool,

    pub mongo_us: u64,
    pub redis_us: u64,
    pub blocks_per_widget: usize,
    pub products_per_list: usize,
    pub port: u16,
}

fn num<T: std::str::FromStr>(key: &str, default: T) -> T {
    env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

impl Config {
    pub fn from_env() -> Self {
        let all_on = env::var("SIM_MODE").as_deref() == Ok("tuned");
        let flag = |key: &str| all_on || env::var(key).as_deref() == Ok("1");

        Self {
            tpl_cache: flag("NJK_CACHE"),
            page_cache: flag("PAGE_CACHE"),
            parallel_fields: flag("PARALLEL_FIELDS"),
            // نفس أزمنة نسخة نود: ٠.٨ms و٠.٣ms
            mongo_us: num("MONGO_US", 800),
            redis_us: num("REDIS_US", 300),
            blocks_per_widget: num("BLOCKS_PER_WIDGET", 3),
            products_per_list: num("PRODUCTS_PER_LIST", 12),
            port: num("PORT", 4200),
        }
    }

    pub fn label(&self) -> String {
        let on: Vec<&str> = [
            self.tpl_cache.then_some("njk"),
            self.page_cache.then_some("page"),
            self.parallel_fields.then_some("parallel"),
        ]
        .into_iter()
        .flatten()
        .collect();

        if on.is_empty() { "asis".to_string() } else { on.join("+") }
    }
}
