use mongodb::{Client, Collection, Database, IndexModel, bson::doc, options::IndexOptions};
use serde::{Serialize, de::DeserializeOwned};

/// تجمّع اتصالات مونجو. الـ Client جوّاه تجمّع داخلي — يُبنى مرة ويُشارَك.
pub struct MongoPool {
    pub client: Client,
    pub db: Database,
}

impl MongoPool {
    pub async fn connect(uri: &str, db_name: &str) -> anyhow::Result<Self> {
        let client = Client::with_uri_str(uri).await?;
        let db = client.database(db_name);
        Ok(Self { client, db })
    }

    pub fn collection<T: Send + Sync + Serialize + DeserializeOwned>(
        &self,
        name: &str,
    ) -> Collection<T> {
        self.db.collection::<T>(name)
    }

    /// كل فهرس مركّب يبدأ بـ store_id — عزل المتاجر مبنيّ في مسار القراءة نفسه.
    pub async fn ensure_indexes(&self) -> anyhow::Result<()> {
        let unique = IndexOptions::builder().unique(true).build();

        self.db
            .collection::<mongodb::bson::Document>("orders")
            .create_index(IndexModel::builder().keys(doc! { "store_id": 1, "placed_at_ms": -1 }).build())
            .await?;

        self.db
            .collection::<mongodb::bson::Document>("stock")
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "store_id": 1, "product_id": 1 })
                    .options(unique)
                    .build(),
            )
            .await?;

        self.db
            .collection::<mongodb::bson::Document>("notifications")
            .create_index(IndexModel::builder().keys(doc! { "store_id": 1, "sent_at_ms": -1 }).build())
            .await?;

        self.db
            .collection::<mongodb::bson::Document>("orders_outbox")
            .create_index(IndexModel::builder().keys(doc! { "published": 1, "occurred_at_ms": 1 }).build())
            .await?;

        Ok(())
    }
}
