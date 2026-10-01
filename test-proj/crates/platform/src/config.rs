use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub mongo_uri: String,
    pub mongo_db: String,
    pub redis_url: String,
    pub amqp_url: String,
    pub bind_addr: String,
    pub openai_api_key: Option<String>,
    pub openai_base: String,
    /// يشغّل مؤلّف الرسائل المعتمد على OpenAI بدل القالب الثابت
    pub use_ai_composer: bool,
}

fn var(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            mongo_uri: var("QUMRA_MONGO_URI", "mongodb://127.0.0.1:27017"),
            mongo_db: var("QUMRA_MONGO_DB", "qumra"),
            redis_url: var("QUMRA_REDIS_URL", "redis://127.0.0.1:6379"),
            amqp_url: var("QUMRA_AMQP_URL", "amqp://guest:guest@127.0.0.1:5672/%2f"),
            bind_addr: var("QUMRA_BIND", "0.0.0.0:3000"),
            openai_api_key: env::var("OPENAI_API_KEY").ok(),
            openai_base: var("OPENAI_BASE", "https://api.openai.com/v1"),
            use_ai_composer: var("QUMRA_AI", "0") == "1",
        })
    }
}
