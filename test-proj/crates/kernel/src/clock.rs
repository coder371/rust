use time::OffsetDateTime;

/// الزمن كاعتمادية صريحة — يجعل منطق الـ domain قابلاً للاختبار بلا ساعة حقيقية.
pub trait Clock: Send + Sync + 'static {
    fn now(&self) -> OffsetDateTime;

    fn now_ms(&self) -> i64 {
        (self.now().unix_timestamp_nanos() / 1_000_000) as i64
    }
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}
