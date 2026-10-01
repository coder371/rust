use crate::domain::{Reservation, StockError, StockItem, StockRepository};
use mongodb::{Collection, bson::doc, options::ReturnDocument};
use qumra_kernel::{ProductId, ReservationId, StoreId};
use qumra_platform::AppState;
use serde::{Deserialize, Serialize};

fn storage<E: std::fmt::Display>(e: E) -> StockError {
    StockError::Storage(e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
struct StockDoc {
    store_id: String,
    product_id: String,
    on_hand: i64,
    reserved: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReservationDoc {
    #[serde(rename = "_id")]
    id: String,
    store_id: String,
    items: Vec<ItemDoc>,
    released: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ItemDoc {
    product_id: String,
    quantity: u32,
}

pub struct MongoStockRepository {
    stock: Collection<StockDoc>,
    reservations: Collection<ReservationDoc>,
}

impl MongoStockRepository {
    pub fn new(state: &AppState) -> Self {
        Self {
            stock: state.mongo.collection::<StockDoc>("stock"),
            reservations: state.mongo.collection::<ReservationDoc>("reservations"),
        }
    }

    /// حجز صنف واحد ذرّياً: التحديث لا يتم إلا إذا كان المتاح كافياً.
    /// شرط `$expr` يجعل الفحص والتحديث عملية واحدة على الخادم،
    /// فلا يوجد سباق بين قراءة الرصيد وكتابته.
    async fn try_reserve_one(
        &self,
        store: &str,
        product: &str,
        qty: u32,
    ) -> Result<bool, StockError> {
        let res = self
            .stock
            .update_one(
                doc! {
                    "store_id": store,
                    "product_id": product,
                    "$expr": { "$gte": [ { "$subtract": ["$on_hand", "$reserved"] }, qty as i64 ] }
                },
                doc! { "$inc": { "reserved": qty as i64 } },
            )
            .await
            .map_err(storage)?;
        Ok(res.modified_count == 1)
    }
}

#[async_trait::async_trait]
impl StockRepository for MongoStockRepository {
    async fn adjust(
        &self,
        store: &StoreId,
        product: &ProductId,
        delta: i64,
    ) -> Result<StockItem, StockError> {
        let doc = self
            .stock
            .find_one_and_update(
                doc! { "store_id": store.as_str(), "product_id": product.as_str() },
                doc! { "$inc": { "on_hand": delta }, "$setOnInsert": { "reserved": 0i64 } },
            )
            .upsert(true)
            .return_document(ReturnDocument::After)
            .await
            .map_err(storage)?
            .ok_or_else(|| StockError::Storage("تعذّر تحديث المخزون".into()))?;

        Ok(StockItem {
            store_id: store.clone(),
            product_id: product.clone(),
            on_hand: doc.on_hand,
            reserved: doc.reserved,
        })
    }

    async fn find(
        &self,
        store: &StoreId,
        product: &ProductId,
    ) -> Result<Option<StockItem>, StockError> {
        let doc = self
            .stock
            .find_one(doc! { "store_id": store.as_str(), "product_id": product.as_str() })
            .await
            .map_err(storage)?;

        Ok(doc.map(|d| StockItem {
            store_id: store.clone(),
            product_id: product.clone(),
            on_hand: d.on_hand,
            reserved: d.reserved,
        }))
    }

    async fn reserve(
        &self,
        store: &StoreId,
        items: &[(ProductId, u32)],
    ) -> Result<Reservation, StockError> {
        let mut done: Vec<ItemDoc> = Vec::new();

        for (product, qty) in items {
            let ok = self
                .try_reserve_one(store.as_str(), product.as_str(), *qty)
                .await?;

            if !ok {
                // تعويض: نتراجع عمّا نجح قبل الفشل
                for d in &done {
                    let _ = self
                        .stock
                        .update_one(
                            doc! { "store_id": store.as_str(), "product_id": &d.product_id },
                            doc! { "$inc": { "reserved": -(d.quantity as i64) } },
                        )
                        .await;
                }
                return Err(StockError::Insufficient(product.to_string()));
            }

            done.push(ItemDoc { product_id: product.to_string(), quantity: *qty });
        }

        let id = ReservationId::generate();
        self.reservations
            .insert_one(ReservationDoc {
                id: id.to_string(),
                store_id: store.to_string(),
                items: done,
                released: false,
            })
            .await
            .map_err(storage)?;

        Ok(Reservation {
            id,
            store_id: store.clone(),
            items: items.to_vec(),
            released: false,
        })
    }

    async fn release(&self, store: &StoreId, id: &ReservationId) -> Result<(), StockError> {
        let r = self
            .reservations
            .find_one(doc! { "_id": id.as_str(), "store_id": store.as_str(), "released": false })
            .await
            .map_err(storage)?
            .ok_or(StockError::ReservationNotFound)?;

        for item in &r.items {
            self.stock
                .update_one(
                    doc! { "store_id": store.as_str(), "product_id": &item.product_id },
                    doc! { "$inc": { "reserved": -(item.quantity as i64) } },
                )
                .await
                .map_err(storage)?;
        }

        self.reservations
            .update_one(
                doc! { "_id": id.as_str() },
                doc! { "$set": { "released": true } },
            )
            .await
            .map_err(storage)?;

        Ok(())
    }
}
