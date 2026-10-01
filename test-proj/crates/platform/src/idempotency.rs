use crate::cache::RedisCache;
use std::sync::Arc;

/// التسليم في RabbitMQ «مرة واحدة على الأقل»، فالمعالجة لازم تكون idempotent.
/// SET NX EX ذرّية: أول من يحجز المفتاح هو الوحيد الذي يعالج الحدث.
pub struct Idempotency {
    cache: Arc<RedisCache>,
}

impl Idempotency {
    pub fn new(cache: Arc<RedisCache>) -> Self {
        Self { cache }
    }

    /// ترجع true لو الحجز نجح (أول مرة نرى هذا الحدث).
    pub async fn claim(&self, event_id: &str, ttl_secs: u64) -> anyhow::Result<bool> {
        let mut conn = self.cache.raw();
        let res: Option<String> = redis::cmd("SET")
            .arg(format!("q:idem:{event_id}"))
            .arg("1")
            .arg("NX")
            .arg("EX")
            .arg(ttl_secs)
            .query_async(&mut conn)
            .await?;
        Ok(res.is_some())
    }
}
