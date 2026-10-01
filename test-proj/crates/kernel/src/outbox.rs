use serde::{Deserialize, Serialize};

/// سجل حدث ينتظر النشر. يُكتب في نفس معاملة مونجو مع الكيان،
/// ثم يلتقطه المُرحِّل في platform وينشره على RabbitMQ.
///
/// الحمولة تُخزَّن كنص JSON عمداً: الـ kernel لا يعرف أنواع الأحداث
/// (تلك تعيش في crate الـ contracts)، والتخزين يبقى محايداً تجاهها.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxRecord {
    pub event_id: String,
    pub routing_key: String,
    pub store_id: String,
    pub occurred_at_ms: i64,
    pub payload_json: String,
}

impl OutboxRecord {
    pub fn new<T: Serialize>(
        routing_key: &str,
        store_id: &str,
        occurred_at_ms: i64,
        payload: &T,
    ) -> serde_json::Result<Self> {
        Ok(Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            routing_key: routing_key.to_string(),
            store_id: store_id.to_string(),
            occurred_at_ms,
            payload_json: serde_json::to_string(payload)?,
        })
    }
}
