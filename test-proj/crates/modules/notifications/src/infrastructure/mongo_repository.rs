use crate::domain::{Notification, NotificationRepository, NotifyError};
use futures::TryStreamExt;
use mongodb::{Collection, bson::doc};
use qumra_kernel::StoreId;
use qumra_platform::AppState;
use serde::{Deserialize, Serialize};

fn storage<E: std::fmt::Display>(e: E) -> NotifyError {
    NotifyError::Storage(e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
struct NotificationDoc {
    #[serde(rename = "_id")]
    id: String,
    store_id: String,
    order_id: String,
    channel: String,
    body: String,
    sent_at_ms: i64,
}

pub struct MongoNotificationRepository {
    coll: Collection<NotificationDoc>,
}

impl MongoNotificationRepository {
    pub fn new(state: &AppState) -> Self {
        Self { coll: state.mongo.collection::<NotificationDoc>("notifications") }
    }
}

#[async_trait::async_trait]
impl NotificationRepository for MongoNotificationRepository {
    async fn record(&self, n: &Notification) -> Result<(), NotifyError> {
        // upsert بمعرّف مشتق من الطلب: طبقة حماية ثانية ضد التكرار
        self.coll
            .replace_one(
                doc! { "_id": &n.id },
                NotificationDoc {
                    id: n.id.clone(),
                    store_id: n.store_id.to_string(),
                    order_id: n.order_id.clone(),
                    channel: n.channel.clone(),
                    body: n.body.clone(),
                    sent_at_ms: n.sent_at_ms,
                },
            )
            .upsert(true)
            .await
            .map_err(storage)?;
        Ok(())
    }

    async fn list(&self, store: &StoreId, limit: i64) -> Result<Vec<Notification>, NotifyError> {
        let docs: Vec<NotificationDoc> = self
            .coll
            .find(doc! { "store_id": store.as_str() })
            .sort(doc! { "sent_at_ms": -1 })
            .limit(limit)
            .await
            .map_err(storage)?
            .try_collect()
            .await
            .map_err(storage)?;

        Ok(docs
            .into_iter()
            .map(|d| Notification {
                id: d.id,
                store_id: StoreId::new(d.store_id),
                order_id: d.order_id,
                channel: d.channel,
                body: d.body,
                sent_at_ms: d.sent_at_ms,
            })
            .collect())
    }
}
