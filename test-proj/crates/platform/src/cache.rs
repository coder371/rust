use redis::AsyncTypedCommands;
use redis::aio::MultiplexedConnection;
use serde::{Serialize, de::DeserializeOwned};

/// Redis ليس مصدر حقيقة أبداً — كل مفتاح هنا قابل لإعادة البناء من مونجو.
pub struct RedisCache {
    conn: MultiplexedConnection,
}

impl RedisCache {
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(url)?;
        Ok(Self {
            conn: client.get_multiplexed_async_connection().await?,
        })
    }

    /// كل مفتاح مسبوق بالمتجر: q:{store}:{...} — لا تسرّب بين المستأجرين
    pub fn key(store: &str, rest: &str) -> String {
        format!("q:{store}:{rest}")
    }

    pub fn raw(&self) -> MultiplexedConnection {
        self.conn.clone()
    }

    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> anyhow::Result<Option<T>> {
        let raw = self.conn.clone().get(key).await?;
        match raw {
            Some(s) => Ok(Some(serde_json::from_str(&s)?)),
            None => Ok(None),
        }
    }

    pub async fn set_json<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_secs: u64,
    ) -> anyhow::Result<()> {
        let s = serde_json::to_string(value)?;
        self.conn.clone().set_ex(key, s, ttl_secs).await?;
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> anyhow::Result<()> {
        self.conn.clone().del(key).await?;
        Ok(())
    }
}
