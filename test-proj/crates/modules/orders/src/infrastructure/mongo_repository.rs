use super::documents::OrderDocument;
use crate::domain::{Order, OrderRepository, RepoError};
use futures::TryStreamExt;
use mongodb::{Client, Collection, bson::doc};
use qumra_kernel::{OrderId, OutboxRecord, StoreId};
use qumra_platform::{AppState, outbox::OutboxDoc};

fn storage<E: std::fmt::Display>(e: E) -> RepoError {
    RepoError::Storage(e.to_string())
}

/// المكان الوحيد في موديول الطلبات الذي يعرف مونجو.
pub struct MongoOrderRepository {
    client: Client, // من AppState — لا اتصال جديد لكل طلب
    orders: Collection<OrderDocument>,
    outbox: Collection<OutboxDoc>,
}

impl MongoOrderRepository {
    pub fn new(state: &AppState) -> Self {
        Self {
            client: state.mongo.client.clone(),
            orders: state.mongo.collection::<OrderDocument>("orders"),
            outbox: state.mongo.collection::<OutboxDoc>("orders_outbox"),
        }
    }

    /// الطلب وأحداثه في معاملة واحدة: يا الاثنان يا ولا واحد.
    async fn write_tx(
        &self,
        order: &Order,
        events: &[OutboxRecord],
        expected: Option<u32>,
    ) -> Result<(), RepoError> {
        let mut session = self.client.start_session().await.map_err(storage)?;
        session.start_transaction().await.map_err(storage)?;

        let doc = OrderDocument::from(order);

        match expected {
            None => {
                self.orders
                    .insert_one(&doc)
                    .session(&mut session)
                    .await
                    .map_err(storage)?;
            }
            Some(v) => {
                let res = self
                    .orders
                    .replace_one(
                        doc! { "_id": &doc.id, "store_id": &doc.store_id, "version": v as i64 },
                        &doc,
                    )
                    .session(&mut session)
                    .await
                    .map_err(storage)?;

                if res.matched_count == 0 {
                    session.abort_transaction().await.map_err(storage)?;
                    return Err(RepoError::VersionConflict);
                }
            }
        }

        if !events.is_empty() {
            let docs: Vec<OutboxDoc> = events.iter().map(OutboxDoc::from).collect();
            self.outbox
                .insert_many(&docs)
                .session(&mut session)
                .await
                .map_err(storage)?;
        }

        session.commit_transaction().await.map_err(storage)?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl OrderRepository for MongoOrderRepository {
    async fn insert(&self, order: &Order, events: &[OutboxRecord]) -> Result<(), RepoError> {
        self.write_tx(order, events, None).await
    }

    async fn save(
        &self,
        order: &Order,
        expected_version: u32,
        events: &[OutboxRecord],
    ) -> Result<(), RepoError> {
        self.write_tx(order, events, Some(expected_version)).await
    }

    async fn find(&self, store: &StoreId, id: &OrderId) -> Result<Option<Order>, RepoError> {
        // store_id في كل مرشّح — الفهرس المركّب { store_id: 1, ... }
        let doc = self
            .orders
            .find_one(doc! { "_id": id.as_str(), "store_id": store.as_str() })
            .await
            .map_err(storage)?;
        Ok(doc.map(Order::from))
    }

    async fn list(&self, store: &StoreId, limit: i64) -> Result<Vec<Order>, RepoError> {
        let docs: Vec<OrderDocument> = self
            .orders
            .find(doc! { "store_id": store.as_str() })
            .sort(doc! { "placed_at_ms": -1 })
            .limit(limit)
            .await
            .map_err(storage)?
            .try_collect()
            .await
            .map_err(storage)?;
        Ok(docs.into_iter().map(Order::from).collect())
    }
}
