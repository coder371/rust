//! الطبقة التحتية المشتركة.
//!
//! كل عميل هنا يُنشأ مرة واحدة عند الإقلاع ويُعاد استخدامه —
//! لا اتصال جديد لكل طلب، ولا قناة جديدة لكل رسالة.
pub mod bus;
pub mod cache;
pub mod config;
pub mod idempotency;
pub mod mongo;
pub mod openai;
pub mod outbox;
pub mod state;

pub use bus::{EventBus, EXCHANGE};
pub use cache::RedisCache;
pub use config::Config;
pub use idempotency::Idempotency;
pub use mongo::MongoPool;
pub use openai::OpenAiClient;
pub use state::AppState;
