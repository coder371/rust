use crate::state::AppState;
use futures::TryStreamExt;
use mongodb::bson::doc;
use qumra_contracts::Envelope;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// شكل السجل داخل مونجو. `_id` هو معرّف الحدث نفسه،
/// فإعادة الإدراج مستحيلة والنشر المكرّر يُكتشف عند المستهلك.
#[derive(Debug, Serialize, Deserialize)]
pub struct OutboxDoc {
    #[serde(rename = "_id")]
    pub event_id: String,
    pub routing_key: String,
    pub store_id: String,
    pub occurred_at_ms: i64,
    pub payload_json: String,
    pub published: bool,
}

impl From<&qumra_kernel::OutboxRecord> for OutboxDoc {
    fn from(r: &qumra_kernel::OutboxRecord) -> Self {
        Self {
            event_id: r.event_id.clone(),
            routing_key: r.routing_key.clone(),
            store_id: r.store_id.clone(),
            occurred_at_ms: r.occurred_at_ms,
            payload_json: r.payload_json.clone(),
            published: false,
        }
    }
}

/// المُرحِّل: يقرأ ما لم يُنشر بعد ويبعثه للناقل ثم يعلّمه منشوراً.
///
/// لماذا هذا موجود أصلاً: لا يمكن الكتابة في مونجو والنشر على RabbitMQ
/// ذرّياً. الحدث يُكتب داخل معاملة الكتابة، وهذه الحلقة تنقله بعدها.
/// النتيجة: قد يُنشر الحدث مرتين، لكنه لا يضيع أبداً.
pub async fn relay(state: AppState, collections: Vec<String>) {
    loop {
        for name in &collections {
            if let Err(e) = drain(&state, name).await {
                tracing::warn!(collection = %name, error = %e, "فشل ترحيل الـ outbox");
            }
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
}

async fn drain(state: &AppState, collection: &str) -> anyhow::Result<usize> {
    let coll = state.mongo.collection::<OutboxDoc>(collection);

    let pending: Vec<OutboxDoc> = coll
        .find(doc! { "published": false })
        .sort(doc! { "occurred_at_ms": 1 })
        .limit(100)
        .await?
        .try_collect()
        .await?;

    let mut sent = 0;
    for rec in pending {
        let envelope = Envelope {
            event_id: rec.event_id.clone(),
            routing_key: rec.routing_key.clone(),
            store_id: rec.store_id.clone(),
            occurred_at_ms: rec.occurred_at_ms,
            payload: serde_json::from_str::<serde_json::Value>(&rec.payload_json)?,
        };

        state
            .bus
            .publish(&rec.routing_key, &serde_json::to_vec(&envelope)?)
            .await?;

        coll.update_one(
            doc! { "_id": &rec.event_id },
            doc! { "$set": { "published": true } },
        )
        .await?;

        tracing::info!(event = %rec.routing_key, id = %rec.event_id, "نُشر");
        sent += 1;
    }
    Ok(sent)
}
