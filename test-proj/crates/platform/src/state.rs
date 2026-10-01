use crate::{
    bus::EventBus, cache::RedisCache, config::Config, idempotency::Idempotency, mongo::MongoPool,
    openai::OpenAiClient,
};
use qumra_kernel::{Clock, SystemClock};
use std::sync::Arc;

/// حاوية الاعتماديات. عملاء تقنيون فقط — لا منطق ولا موديولات.
/// كلها Arc فالـ clone رخيص، وكلها تُبنى مرة واحدة في `connect`.
#[derive(Clone)]
pub struct AppState {
    pub mongo: Arc<MongoPool>,
    pub cache: Arc<RedisCache>,
    pub bus: Arc<EventBus>,
    pub ai: Arc<OpenAiClient>,
    pub idempotency: Arc<Idempotency>,
    pub config: Arc<Config>,
    pub clock: Arc<dyn Clock>,
}

impl AppState {
    pub async fn connect(cfg: &Config) -> anyhow::Result<Self> {
        let mongo = MongoPool::connect(&cfg.mongo_uri, &cfg.mongo_db).await?;
        mongo.ensure_indexes().await?;
        tracing::info!(db = %cfg.mongo_db, "mongodb متصل");

        let cache = Arc::new(RedisCache::connect(&cfg.redis_url).await?);
        tracing::info!("redis متصل");

        let bus = EventBus::connect(&cfg.amqp_url).await?;
        tracing::info!(exchange = crate::bus::EXCHANGE, "rabbitmq متصل");

        Ok(Self {
            mongo: Arc::new(mongo),
            idempotency: Arc::new(Idempotency::new(cache.clone())),
            cache,
            bus: Arc::new(bus),
            ai: Arc::new(OpenAiClient::new(cfg)?),
            config: Arc::new(cfg.clone()),
            clock: Arc::new(SystemClock),
        })
    }
}
