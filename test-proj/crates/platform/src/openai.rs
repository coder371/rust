use crate::config::Config;
use serde_json::json;
use std::time::Duration;

/// عميل reqwest واحد للعملية كلها: تجمّع اتصالات + keep-alive + مهلة.
/// بناء عميل جديد لكل نداء يفقد التجمّع ويضاعف زمن مصافحة TLS.
pub struct OpenAiClient {
    http: reqwest::Client,
    base: String,
    api_key: Option<String>,
}

impl OpenAiClient {
    pub fn new(cfg: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(20))
                .build()?,
            base: cfg.openai_base.clone(),
            api_key: cfg.openai_api_key.clone(),
        })
    }

    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }

    pub async fn complete(&self, system: &str, user: &str) -> anyhow::Result<String> {
        let key = self
            .api_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("OPENAI_API_KEY غير مضبوط"))?;

        let res: serde_json::Value = self
            .http
            .post(format!("{}/chat/completions", self.base))
            .bearer_auth(key)
            .json(&json!({
                "model": "gpt-4o-mini",
                "messages": [
                    { "role": "system", "content": system },
                    { "role": "user",   "content": user   }
                ]
            }))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(res["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .to_string())
    }
}
